import init, { run_code, format_code } from './wasm/loft_wasm';

init()
  .then(() => self.postMessage({ id: '__ready' }))
  .catch(err => self.postMessage({ id: '__ready', error: String(err) }));

self.onmessage = ({ data }) => {
  const { id, type, code } = data;
  try {
    let result;
    if (type === 'run') {
      result = run_code(code);
    } else if (type === 'format') {
      result = format_code(code);
    }
    self.postMessage({ id, result });
  } catch (err) {
    // Recoverable JS-level error (shouldn't normally happen; panics crash the worker)
    self.postMessage({ id, error: String(err) });
  }
};
