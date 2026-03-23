(function runnableExamples() {
    function decodeBase64Utf8(value) {
        const binary = window.atob(value);
        const bytes = Uint8Array.from(binary, char => char.charCodeAt(0));
        return new TextDecoder().decode(bytes);
    }

    function shuffle(array) {
        for (let i = array.length - 1; i > 0; i -= 1) {
            const j = Math.floor(Math.random() * (i + 1));
            const tmp = array[i];
            array[i] = array[j];
            array[j] = tmp;
        }
    }

    Array.from(document.querySelectorAll('pre')).forEach(function(preBlock) {
        const codeBlock = preBlock.querySelector('code.runnable-example');
        if (!codeBlock) return;

        const outputData = preBlock.nextElementSibling;
        if (!outputData || !outputData.classList.contains('runnable-example-output')) return;

        const payload = JSON.parse(outputData.textContent);
        const outputs = Array.isArray(payload.outputs_base64) ? payload.outputs_base64 : [];
        const delay = Number(payload.delay_ms || 180);
        const isNondeterministic = payload.mode === 'nondeterministic';
        let pendingTimer = null;
        let resultBlock = null;
        let lastOutputIndex = null;
        let outputOrder = [];

        function ensureResultBlock() {
            if (!resultBlock) {
                resultBlock = preBlock.querySelector('.result');
            }
            if (!resultBlock) {
                resultBlock = document.createElement('code');
                resultBlock.className = 'result hljs language-bash';
                preBlock.append(resultBlock);
            }
            return resultBlock;
        }

        function refillOutputOrder() {
            outputOrder = outputs.map(function(_, index) {
                return index;
            });
            shuffle(outputOrder);

            if (outputOrder.length > 1 && lastOutputIndex !== null && outputOrder[0] === lastOutputIndex) {
                const swapIndex = outputOrder.findIndex(function(index) {
                    return index !== lastOutputIndex;
                });
                if (swapIndex > 0) {
                    const tmp = outputOrder[0];
                    outputOrder[0] = outputOrder[swapIndex];
                    outputOrder[swapIndex] = tmp;
                }
            }
        }

        function nextOutputText() {
            if (outputs.length === 0) {
                return 'No output';
            }

            if (!isNondeterministic) {
                return decodeBase64Utf8(outputs[0] || '');
            }

            if (outputOrder.length === 0) {
                refillOutputOrder();
            }

            const nextIndex = outputOrder.shift();
            lastOutputIndex = nextIndex;
            return decodeBase64Utf8(outputs[nextIndex] || '');
        }

        let buttons = preBlock.querySelector('.buttons');
        if (!buttons) {
            buttons = document.createElement('div');
            buttons.className = 'buttons';
            preBlock.insertBefore(buttons, preBlock.firstChild);
        }

        const runCodeButton = document.createElement('button');
        runCodeButton.className = 'play-button';
        runCodeButton.title = 'Run this code';
        runCodeButton.setAttribute('aria-label', runCodeButton.title);
        runCodeButton.innerHTML = document.getElementById('fa-play').innerHTML;
        buttons.appendChild(runCodeButton);

        runCodeButton.addEventListener('click', function() {
            if (pendingTimer !== null) {
                window.clearTimeout(pendingTimer);
            }

            const outputBlock = ensureResultBlock();
            outputBlock.classList.remove('result-no-output');
            outputBlock.innerText = 'Running...';

            pendingTimer = window.setTimeout(function() {
                const stdout = nextOutputText();
                if (stdout.trim() === '' || stdout === 'No output') {
                    outputBlock.innerText = 'No output';
                    outputBlock.classList.add('result-no-output');
                } else {
                    outputBlock.innerText = stdout;
                    outputBlock.classList.remove('result-no-output');
                }
                pendingTimer = null;
            }, delay);
        });
    });
})();
