import * as wasm from "gwendr";

const program = wasm.setup();

const mainLoop = () => {
  program.update();
  requestAnimationFrame(mainLoop);
}

requestAnimationFrame(mainLoop);

window.addEventListener('keydown', event => {
  program.handle_key_down(event.key);
  event.preventDefault();
});
