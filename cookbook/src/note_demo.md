# Note Demo

This page exists to demonstrate the custom note styles available in the book.

## Note

{{#note note}}
Use a standard note for neutral context, background, or a general heads-up that supports the surrounding explanation.
{{/note}}

## Tip

{{#note tip}}
Use a tip for practical shortcuts, smoother workflows, or small habits that make the reader's life easier.
{{/note}}

## Important

{{#note important}}
Use an important note when the reader should pay close attention because a concept or requirement is easy to miss.
{{/note}}

## Warning

{{#note warning}}
Use a warning when a mistake is likely to cause confusion, invalid output, or time-consuming debugging.
{{/note}}

## Caution

{{#note caution}}
Use a caution note when the reader could expose sensitive material, lose funds, or take a risk that deserves extra care.
{{/note}}

## Authoring

The helper syntax is:

```md
{{#note tip}}
Use `cargo add bitcoin --features rand-std` if you want random key generation.
{{/note}}
```

You can also override the default title:

```md
{{#note note title="Runnable Examples"}}
This note uses a custom title while keeping the base note style.
{{/note}}
```

Raw HTML still works if you want full manual control, but the helper syntax should cover most cases.
