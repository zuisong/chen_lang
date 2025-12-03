import init, { run_wasm } from './pkg/chen_lang.js';

async function run() {
    await init();
    const runBtn = document.getElementById('run');
    const codeArea = document.getElementById('code');
    const outputArea = document.getElementById('output');

    runBtn.addEventListener('click', () => {
        const code = codeArea.value;
        try {
            const result = run_wasm(code);
            outputArea.textContent = result;
        } catch (e) {
            outputArea.textContent = `Error: ${e}`;
        }
    });
}

run();
