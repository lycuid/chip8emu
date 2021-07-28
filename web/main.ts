const WIDTH = 64;
const HEIGHT = 32;
const SCALE = 10;
let RUNNING = false;

const RomOptions = [{ name: "PONG2" }, { name: "INVADERS" }, { name: "BLITZ" }];

const romselect = document.getElementById("romselect");
const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const context = canvas.getContext("2d");

canvas.width = WIDTH * SCALE;
canvas.height = HEIGHT * SCALE;

const clear_canvas = () => {
  // @ts-ignore
  context?.fillStyle = "black";
  context?.fillRect(0, 0, WIDTH * SCALE, HEIGHT * SCALE);
  // @ts-ignore
  context?.fillStyle = "white";
};

const render_on_canvas = (frame_buffer: Uint8Array) => {
  clear_canvas();

  for (let addr = 0; addr < frame_buffer.length; addr++) {
    const x = (addr % WIDTH) * SCALE;
    const y = Math.floor(addr / WIDTH) * SCALE;
    const pixel = frame_buffer[addr];

    if (pixel) context?.fillRect(x, y, SCALE, SCALE);
  }
};

(async () => {
  const lib = await (await import("../pkg")).default;
  const cpu = new lib.Chip8Cpu();

  clear_canvas();

  const game_loop = () => {
    cpu.emulate_cycle();

    if (cpu.frame_buffer_updated())
      // heavy process "FOR JAVASCRIPT", draw only if updated.
      render_on_canvas(cpu.frame_buffer());

    if (RUNNING)
      // This is load different rom without refreshing the page.
      requestAnimationFrame(game_loop);
  };

  for (const romoption of RomOptions) {
    const option = document.createElement("option");
    option.value = option.innerText = romoption.name;
    romselect?.appendChild(option);
  }

  romselect?.addEventListener("change", async (e) => {
    const { value } = e.target as HTMLInputElement;

    RUNNING = false;
    if (!value.length) {
      clear_canvas();
      return;
    }

    const response = await fetch(`roms/${value}`);
    if (!response.ok) return;

    const rom = new Uint8Array(await response.arrayBuffer());
    cpu.load(rom);
    RUNNING = true;
    game_loop();
  });
})();
