use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use pulldown_cmark::{html, Options, Parser};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use tempfile::tempdir;

const PREPROCESSOR_NAME: &str = "runnable-examples";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let preprocessor = RunnableExamples::new();

    if args.get(1).map(String::as_str) == Some("supports") {
        let renderer = args.get(2).map(String::as_str).unwrap_or("");
        let supported = preprocessor.supports_renderer(renderer).unwrap_or(false);
        process::exit(if supported { 0 } else { 1 });
    }

    if let Err(err) = handle_preprocessing(&preprocessor) {
        eprintln!("{err:?}");
        process::exit(1);
    }
}

fn handle_preprocessing(preprocessor: &dyn Preprocessor) -> Result<()> {
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;
    let processed = preprocessor.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed)?;
    Ok(())
}

struct RunnableExamples;

#[derive(Debug, Clone)]
struct Settings {
    enabled: bool,
    examples_dir: PathBuf,
    output_dir: PathBuf,
    manifest_path: PathBuf,
    target_dir: PathBuf,
    fake_running_ms: u64,
    nondeterministic_sample_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum ExampleMode {
    Auto,
    Deterministic,
    Nondeterministic,
}

impl ExampleMode {
    fn from_token(token: &str) -> Result<Self> {
        match token {
            "auto" => Ok(Self::Auto),
            "deterministic" => Ok(Self::Deterministic),
            "nondeterministic" => Ok(Self::Nondeterministic),
            other => bail!("unsupported runnable example mode `{other}`"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Deterministic => "deterministic",
            Self::Nondeterministic => "nondeterministic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RunnableDirective {
    rel_path: PathBuf,
    mode: ExampleMode,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RunnableArtifact {
    mode: ExampleMode,
    source_hash: String,
    outputs: Vec<String>,
}

#[derive(Debug, Clone)]
struct NoteDirective {
    note_type: NoteType,
    title: String,
    body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoteType {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl NoteType {
    fn from_token(token: &str) -> Result<Self> {
        match token {
            "note" => Ok(Self::Note),
            "tip" => Ok(Self::Tip),
            "important" => Ok(Self::Important),
            "warning" => Ok(Self::Warning),
            "caution" => Ok(Self::Caution),
            other => bail!("unsupported note type `{other}`"),
        }
    }

    fn as_class_suffix(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Tip => "tip",
            Self::Important => "important",
            Self::Warning => "warning",
            Self::Caution => "caution",
        }
    }

    fn default_title(self) -> &'static str {
        match self {
            Self::Note => "Note",
            Self::Tip => "Tip",
            Self::Important => "Important",
            Self::Warning => "Warning",
            Self::Caution => "Caution",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct FenceDelimiter {
    ch: char,
    len: usize,
}

impl Settings {
    fn from_ctx(ctx: &PreprocessorContext) -> Result<Self> {
        Ok(Self {
            enabled: ctx
                .config
                .get::<bool>("preprocessor.runnable-examples.enabled")?
                .unwrap_or(false),
            examples_dir: PathBuf::from(
                ctx.config
                    .get::<String>("preprocessor.runnable-examples.examples-dir")?
                    .unwrap_or_else(|| "runnable_examples/examples".to_string()),
            ),
            output_dir: PathBuf::from(
                ctx.config
                    .get::<String>("preprocessor.runnable-examples.output-dir")?
                    .unwrap_or_else(|| "runnable_examples/output".to_string()),
            ),
            manifest_path: PathBuf::from(
                ctx.config
                    .get::<String>("preprocessor.runnable-examples.manifest-path")?
                    .unwrap_or_else(|| "runnable_examples/Cargo.toml".to_string()),
            ),
            target_dir: PathBuf::from(
                ctx.config
                    .get::<String>("preprocessor.runnable-examples.target-dir")?
                    .unwrap_or_else(|| "/tmp/rust-bitcoin-cookbook-runnable-target".to_string()),
            ),
            fake_running_ms: ctx
                .config
                .get::<u64>("preprocessor.runnable-examples.fake-running-ms")?
                .unwrap_or(180),
            nondeterministic_sample_count: ctx
                .config
                .get::<usize>("preprocessor.runnable-examples.nondeterministic-sample-count")?
                .unwrap_or(10)
                .max(1),
        })
    }
}

impl RunnableExamples {
    fn new() -> Self {
        Self
    }
}

impl Preprocessor for RunnableExamples {
    fn name(&self) -> &str {
        PREPROCESSOR_NAME
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let settings = Settings::from_ctx(ctx)?;
        let referenced_examples = collect_referenced_examples(&book)?;

        if settings.enabled {
            for directive in &referenced_examples {
                ensure_artifact(ctx.root.as_path(), &settings, directive)?;
            }
        }

        process_items(&mut book.items, ctx.root.as_path(), &settings)?;
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool> {
        Ok(renderer == "html")
    }
}

fn process_items(items: &mut [BookItem], root: &Path, settings: &Settings) -> Result<()> {
    for item in items {
        if let BookItem::Chapter(chapter) = item {
            process_chapter(chapter, root, settings)?;
            process_items(&mut chapter.sub_items, root, settings)?;
        }
    }
    Ok(())
}

fn note_header_regex() -> Regex {
    Regex::new(r#"^([a-z]+)(?:\s+title="([^"]+)")?\s*$"#).expect("valid note header regex")
}

fn package_name_regex() -> Regex {
    Regex::new(r#"(?m)^name\s*=\s*"[^"]+"\s*$"#).expect("valid cargo package name regex")
}

fn parse_runnable_directive_body(body: &str) -> Result<RunnableDirective> {
    let mut parts = body.split_whitespace();
    let rel_path = PathBuf::from(parts.next().context("missing runnable path")?);
    let mut mode = ExampleMode::Auto;

    for token in parts {
        if let Some(value) = token.strip_prefix("mode=") {
            mode = ExampleMode::from_token(value)?;
        } else if token == "auto" {
            mode = ExampleMode::Auto;
        } else if token == "nondeterministic" {
            mode = ExampleMode::Nondeterministic;
        } else if token == "deterministic" {
            mode = ExampleMode::Deterministic;
        } else {
            bail!("unsupported runnable example option `{token}`");
        }
    }

    Ok(RunnableDirective { rel_path, mode })
}

fn parse_note_directive_header(body: &str) -> Result<NoteDirective> {
    let captures = note_header_regex()
        .captures(body)
        .with_context(|| format!("invalid note directive `{body}`"))?;

    let note_type = NoteType::from_token(captures.get(1).context("missing note type")?.as_str())?;
    let title = captures
        .get(2)
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| note_type.default_title().to_string());

    Ok(NoteDirective {
        note_type,
        title,
        body: String::new(),
    })
}

fn collect_referenced_examples(book: &Book) -> Result<Vec<RunnableDirective>> {
    let mut examples = BTreeMap::new();

    for chapter in book.chapters() {
        for directive in find_runnable_directives(&chapter.content)? {
            if let Some(previous_mode) = examples.get_mut(&directive.rel_path) {
                *previous_mode =
                    merge_example_modes(*previous_mode, directive.mode).with_context(|| {
                        format!(
                            "runnable example {} is referenced with conflicting modes",
                            directive.rel_path.display()
                        )
                    })?;
            } else {
                examples.insert(directive.rel_path.clone(), directive.mode);
            }
        }
    }

    Ok(examples
        .into_iter()
        .map(|(rel_path, mode)| RunnableDirective { rel_path, mode })
        .collect())
}

fn merge_example_modes(existing: ExampleMode, incoming: ExampleMode) -> Result<ExampleMode> {
    match (existing, incoming) {
        (left, right) if left == right => Ok(left),
        (ExampleMode::Auto, other) | (other, ExampleMode::Auto) => Ok(other),
        _ => bail!("conflicting explicit runnable modes"),
    }
}

fn find_runnable_directives(content: &str) -> Result<Vec<RunnableDirective>> {
    let mut directives = Vec::new();
    let mut lines = content.split_inclusive('\n').peekable();
    let mut active_fence: Option<FenceDelimiter> = None;

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if let Some(fence) = active_fence {
            if is_matching_fence_close(trimmed, fence) {
                active_fence = None;
            }
            continue;
        }

        if let Some(fence) = parse_fence_delimiter(trimmed) {
            active_fence = Some(fence);
            continue;
        }

        if let Some(body) = parse_runnable_line(trimmed) {
            directives.push(parse_runnable_directive_body(body)?);
            continue;
        }

        if parse_note_start_line(trimmed).is_some() {
            consume_note_body(&mut lines)?;
        }
    }

    Ok(directives)
}

fn process_chapter(chapter: &mut Chapter, root: &Path, settings: &Settings) -> Result<()> {
    chapter.content = expand_directives(&chapter.content, root, settings)?;
    Ok(())
}

fn expand_directives(content: &str, root: &Path, settings: &Settings) -> Result<String> {
    let mut output = String::with_capacity(content.len());
    let mut lines = content.split_inclusive('\n').peekable();
    let mut active_fence: Option<FenceDelimiter> = None;

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if let Some(fence) = active_fence {
            output.push_str(line);
            if is_matching_fence_close(trimmed, fence) {
                active_fence = None;
            }
            continue;
        }

        if let Some(fence) = parse_fence_delimiter(trimmed) {
            output.push_str(line);
            active_fence = Some(fence);
            continue;
        }

        if let Some(body) = parse_runnable_line(trimmed) {
            let directive = parse_runnable_directive_body(body)?;
            output.push_str(&render_runnable_block(root, settings, &directive)?);
            continue;
        }

        if let Some(header_body) = parse_note_start_line(trimmed) {
            let mut directive = parse_note_directive_header(header_body)?;
            directive.body = collect_note_body(&mut lines)?;
            output.push_str(&render_note_block(&directive)?);
            continue;
        }

        output.push_str(line);
    }

    Ok(output)
}

fn parse_runnable_line(line: &str) -> Option<&str> {
    line.strip_prefix("{{#runnable ")
        .and_then(|body| body.strip_suffix("}}"))
        .map(str::trim)
}

fn parse_note_start_line(line: &str) -> Option<&str> {
    line.strip_prefix("{{#note ")
        .and_then(|body| body.strip_suffix("}}"))
        .map(str::trim)
}

fn consume_note_body<'a, I>(lines: &mut I) -> Result<()>
where
    I: Iterator<Item = &'a str>,
{
    while let Some(line) = lines.next() {
        if line.trim() == "{{/note}}" {
            return Ok(());
        }
    }

    bail!("unterminated note block")
}

fn collect_note_body<'a, I>(lines: &mut I) -> Result<String>
where
    I: Iterator<Item = &'a str>,
{
    let mut body = String::new();

    while let Some(line) = lines.next() {
        if line.trim() == "{{/note}}" {
            return Ok(body);
        }
        body.push_str(line);
    }

    bail!("unterminated note block")
}

fn parse_fence_delimiter(line: &str) -> Option<FenceDelimiter> {
    let mut chars = line.chars();
    let ch = chars.next()?;
    if ch != '`' && ch != '~' {
        return None;
    }

    let len = line
        .chars()
        .take_while(|candidate| *candidate == ch)
        .count();
    if len < 3 {
        return None;
    }

    Some(FenceDelimiter { ch, len })
}

fn is_matching_fence_close(line: &str, fence: FenceDelimiter) -> bool {
    parse_fence_delimiter(line)
        .map(|candidate| candidate.ch == fence.ch && candidate.len >= fence.len)
        .unwrap_or(false)
}

fn render_runnable_block(
    root: &Path,
    settings: &Settings,
    directive: &RunnableDirective,
) -> Result<String> {
    let source_path = root.join(&directive.rel_path);
    let source = fs::read_to_string(&source_path)
        .with_context(|| format!("unable to read runnable example {}", source_path.display()))?;
    let display_source = render_display_source(&source);

    let mut rendered = String::new();
    rendered.push_str("```rust");
    if settings.enabled {
        rendered.push_str(",runnable-example");
    }
    rendered.push('\n');
    rendered.push_str(&display_source);
    if !display_source.ends_with('\n') {
        rendered.push('\n');
    }
    rendered.push_str("```\n");

    if settings.enabled {
        let artifact = read_artifact_for_directive(root, settings, directive)?;
        let payload = json!({
            "outputs_base64": artifact
                .outputs
                .iter()
                .map(|output| BASE64.encode(output.as_bytes()))
                .collect::<Vec<_>>(),
            "delay_ms": settings.fake_running_ms,
            "mode": artifact.mode.as_str(),
        });
        rendered.push_str("<script type=\"application/json\" class=\"runnable-example-output\">\n");
        rendered.push_str(&payload.to_string());
        rendered.push_str("\n</script>");
    }

    Ok(rendered)
}

fn render_note_block(directive: &NoteDirective) -> Result<String> {
    let body_markdown = directive.body.trim();
    if body_markdown.is_empty() {
        bail!("note block `{}` cannot be empty", directive.title);
    }

    let body_html = render_markdown_fragment(body_markdown);
    let mut rendered = String::new();
    rendered.push_str(&format!(
        "<div class=\"book-note book-note-{}\">\n",
        directive.note_type.as_class_suffix()
    ));
    rendered.push_str(&format!(
        "<p class=\"book-note-title\">{}</p>\n",
        escape_html(&directive.title)
    ));
    rendered.push_str(&body_html);
    if !body_html.ends_with('\n') {
        rendered.push('\n');
    }
    rendered.push_str("</div>\n");
    Ok(rendered)
}

fn render_markdown_fragment(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(markdown, options);
    let mut rendered = String::new();
    html::push_html(&mut rendered, parser);
    rendered
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn render_display_source(source: &str) -> String {
    let mut rendered = String::new();

    for chunk in source.split_inclusive('\n') {
        let has_newline = chunk.ends_with('\n');
        let line = chunk.strip_suffix('\n').unwrap_or(chunk);

        if line == "#" || line.starts_with("# ") {
            continue;
        }

        rendered.push_str(line);

        if has_newline {
            rendered.push('\n');
        }
    }

    if !source.ends_with('\n') && rendered.ends_with('\n') {
        rendered.pop();
    }

    rendered
}

fn render_execution_source(source: &str) -> String {
    let mut rendered = String::new();

    for chunk in source.split_inclusive('\n') {
        let has_newline = chunk.ends_with('\n');
        let line = chunk.strip_suffix('\n').unwrap_or(chunk);

        if line == "#" {
            continue;
        }

        if let Some(stripped) = line.strip_prefix("# ") {
            rendered.push_str(stripped);
        } else {
            rendered.push_str(line);
        }

        if has_newline {
            rendered.push('\n');
        }
    }

    if !source.ends_with('\n') && rendered.ends_with('\n') {
        rendered.pop();
    }

    rendered
}

fn read_artifact_for_directive(
    root: &Path,
    settings: &Settings,
    directive: &RunnableDirective,
) -> Result<RunnableArtifact> {
    let output_path = resolve_path(
        root,
        &artifact_output_path_for(settings, &directive.rel_path),
    );
    read_artifact(&output_path)
        .with_context(|| format!("unable to read runnable artifact {}", output_path.display()))
}

fn ensure_artifact(root: &Path, settings: &Settings, directive: &RunnableDirective) -> Result<()> {
    let example_path = root.join(&directive.rel_path);
    let source = fs::read_to_string(&example_path)
        .with_context(|| format!("unable to read runnable example {}", example_path.display()))?;
    let source_hash = source_hash(&source);
    let output_path = resolve_path(
        root,
        &artifact_output_path_for(settings, &directive.rel_path),
    );

    if let Ok(existing) = read_artifact(&output_path) {
        if artifact_is_current(&existing, directive, &source_hash, settings) {
            return Ok(());
        }
    }

    let manifest_path = resolve_path(root, &settings.manifest_path);
    let manifest_template = fs::read_to_string(&manifest_path).with_context(|| {
        format!(
            "unable to read runnable manifest template {}",
            manifest_path.display()
        )
    })?;
    let artifact = generate_artifact(
        root,
        settings,
        directive,
        &source,
        &manifest_template,
        source_hash,
    )?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_path, serde_json::to_string_pretty(&artifact)?).with_context(|| {
        format!(
            "unable to write runnable artifact {}",
            output_path.display()
        )
    })?;

    Ok(())
}

fn read_artifact(output_path: &Path) -> Result<RunnableArtifact> {
    let artifact_text = fs::read_to_string(output_path)?;
    serde_json::from_str(&artifact_text).with_context(|| {
        format!(
            "unable to parse runnable artifact {}",
            output_path.display()
        )
    })
}

fn artifact_is_current(
    artifact: &RunnableArtifact,
    directive: &RunnableDirective,
    source_hash: &str,
    settings: &Settings,
) -> bool {
    artifact.source_hash == source_hash
        && artifact_has_valid_shape(artifact, settings)
        && requested_mode_accepts_artifact_mode(directive.mode, artifact.mode)
}

fn artifact_has_valid_shape(artifact: &RunnableArtifact, settings: &Settings) -> bool {
    match artifact.mode {
        ExampleMode::Auto => false,
        ExampleMode::Deterministic => artifact.outputs.len() == 1,
        ExampleMode::Nondeterministic => {
            artifact.outputs.len() == settings.nondeterministic_sample_count
        }
    }
}

fn requested_mode_accepts_artifact_mode(
    requested_mode: ExampleMode,
    artifact_mode: ExampleMode,
) -> bool {
    match requested_mode {
        ExampleMode::Auto => matches!(
            artifact_mode,
            ExampleMode::Deterministic | ExampleMode::Nondeterministic
        ),
        ExampleMode::Deterministic => artifact_mode == ExampleMode::Deterministic,
        ExampleMode::Nondeterministic => artifact_mode == ExampleMode::Nondeterministic,
    }
}

fn generate_artifact(
    root: &Path,
    settings: &Settings,
    directive: &RunnableDirective,
    source: &str,
    manifest_template: &str,
    source_hash: String,
) -> Result<RunnableArtifact> {
    match directive.mode {
        ExampleMode::Auto => generate_auto_artifact(
            root,
            settings,
            directive,
            source,
            manifest_template,
            source_hash,
        ),
        ExampleMode::Deterministic => Ok(RunnableArtifact {
            mode: ExampleMode::Deterministic,
            source_hash,
            outputs: vec![run_example_once(
                root,
                settings,
                &directive.rel_path,
                source,
                manifest_template,
            )?],
        }),
        ExampleMode::Nondeterministic => {
            let mut outputs = Vec::with_capacity(settings.nondeterministic_sample_count);
            for _ in 0..settings.nondeterministic_sample_count {
                outputs.push(run_example_once(
                    root,
                    settings,
                    &directive.rel_path,
                    source,
                    manifest_template,
                )?);
            }

            Ok(RunnableArtifact {
                mode: ExampleMode::Nondeterministic,
                source_hash,
                outputs,
            })
        }
    }
}

fn generate_auto_artifact(
    root: &Path,
    settings: &Settings,
    directive: &RunnableDirective,
    source: &str,
    manifest_template: &str,
    source_hash: String,
) -> Result<RunnableArtifact> {
    let sample_count = settings.nondeterministic_sample_count.max(1);
    let mut outputs = Vec::with_capacity(sample_count);
    for _ in 0..sample_count {
        outputs.push(run_example_once(
            root,
            settings,
            &directive.rel_path,
            source,
            manifest_template,
        )?);
    }

    let inferred_mode = if outputs.iter().skip(1).all(|output| output == &outputs[0]) {
        outputs.truncate(1);
        ExampleMode::Deterministic
    } else {
        ExampleMode::Nondeterministic
    };

    Ok(RunnableArtifact {
        mode: inferred_mode,
        source_hash,
        outputs,
    })
}

fn example_package_name(rel_path: &Path) -> String {
    let without_extension = rel_path.with_extension("");
    let mut package_name = String::from("runnable-");
    let mut last_was_dash = true;

    for ch in without_extension.to_string_lossy().chars() {
        if ch.is_ascii_alphanumeric() {
            package_name.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            package_name.push('-');
            last_was_dash = true;
        }
    }

    while package_name.ends_with('-') {
        package_name.pop();
    }

    if package_name == "runnable" {
        package_name.push_str("-example");
    }

    package_name
}

fn render_manifest_for_example(manifest_template: &str, rel_path: &Path) -> Result<String> {
    let package_name = example_package_name(rel_path);
    let replaced =
        package_name_regex().replacen(manifest_template, 1, format!("name = \"{package_name}\""));

    if replaced == manifest_template {
        bail!("unable to rewrite package name in runnable manifest template");
    }

    Ok(replaced.into_owned())
}

fn run_example_once(
    root: &Path,
    settings: &Settings,
    rel_path: &Path,
    source: &str,
    manifest_template: &str,
) -> Result<String> {
    let tmp = tempdir().context("unable to create tempdir for runnable example")?;
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir)?;
    let example_manifest = render_manifest_for_example(manifest_template, rel_path)?;
    fs::write(tmp.path().join("Cargo.toml"), example_manifest)?;
    let execution_source = render_execution_source(source);
    fs::write(src_dir.join("main.rs"), execution_source)?;

    let command_output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(tmp.path().join("Cargo.toml"))
        .arg("--target-dir")
        .arg(resolve_path(root, &settings.target_dir))
        .output()
        .with_context(|| format!("failed to execute cargo for {}", rel_path.display()))?;

    if !command_output.status.success() {
        bail!(
            "runnable example {} failed with status {}\nstdout:\n{}\nstderr:\n{}",
            rel_path.display(),
            command_output.status,
            String::from_utf8_lossy(&command_output.stdout),
            String::from_utf8_lossy(&command_output.stderr)
        );
    }

    if !command_output.stderr.is_empty() {
        bail!(
            "runnable example {} wrote to stderr:\n{}",
            rel_path.display(),
            String::from_utf8_lossy(&command_output.stderr)
        );
    }

    String::from_utf8(command_output.stdout).with_context(|| {
        format!(
            "runnable example {} wrote non-UTF-8 stdout",
            rel_path.display()
        )
    })
}

fn artifact_output_path_for(settings: &Settings, rel_path: &Path) -> PathBuf {
    let relative = rel_path
        .strip_prefix(&settings.examples_dir)
        .unwrap_or(rel_path);
    let mut output_path = settings.output_dir.join(relative);
    output_path.set_extension("samples.json");
    output_path
}

fn source_hash(source: &str) -> String {
    let digest = Sha256::digest(source.as_bytes());
    let mut rendered = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut rendered, "{byte:02x}").expect("writing to string cannot fail");
    }
    rendered
}

fn resolve_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}
