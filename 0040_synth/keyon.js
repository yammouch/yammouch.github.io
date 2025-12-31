import { initSync, Source } from "./engine/pkg/glue.js";

class SquareProcessor extends AudioWorkletProcessor {

  constructor() {
    super();
    this.wasm = null;
    this.port.onmessage = (e) => {
      console.log(e.data);
      if (e.data.cmd == "on") {
        console.log(e.data.note);
        this.src.on(e.data.note);
        this.port.postMessage( {chord_root: this.src.crt() } );
      } else if (e.data.cmd == "off") {
        this.src.off(e.data.note);
      } else if (e.data.cmd == "init") {
        this.wasm = initSync(e.data.wasm);
        this.src = Source.new(e.data.master / e.data.sampleRate);
      } else if (e.data.cmd == "harm") {
        console.log(e.data);
        this.src.harm(e.data.harm, e.data.value*0.125);
      }
    };
  }

  process(inputs, outputs, parameters) {
    if (!this.wasm) { return true; }
    const output = outputs[0];
    const channel = output[0];

    this.src.tick(channel.length);
    const out_ptr = this.src.ptr();
    const f32view = new Float32Array(
     this.wasm.memory.buffer, out_ptr, channel.length);
    channel.set(f32view);

    return true;
  }

}

registerProcessor("sq-pr", SquareProcessor);
