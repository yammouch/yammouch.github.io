export class RingBuf {

  constructor(arg0, arg1, arg2) {
    if (typeof arg0 === 'number') {
      let n = 1 << (31 - Math.clz32(arg0));
      const u32n = Uint32Array.BYTES_PER_ELEMENT;
      let sab = new SharedArrayBuffer((n + 2) * u32n);
      this.data = new Uint32Array(sab, 0           , n);
      this.wptr = new Uint32Array(sab, u32n *  n   , 1);
      this.rptr = new Uint32Array(sab, u32n * (n+1), 1);
      this.mask = n - 1;
    } else {
      this.data = arg0; // Uint32Array assumed
      this.wptr = arg1; // Uint32Array assumed
      this.rptr = arg2; // Uint32Array assumed
      this.mask = (1 << (31 - Math.clz32(this.data.length))) - 1;
    }
  }

  push(v) {
    let i = this.wptr[0];
    this.data[i] = v;
    Atomics.store(this.wptr, 0, (i + 1)&this.mask);
  }

  pop() {
    let i = Atomics.load(this.wptr, 0);
    if (i === this.rptr[0]) {
      return null;
    } else {
      const rv = this.data[this.rptr[0]];
      this.rptr[0] = (this.rptr[0] + 1)&this.mask;
      return rv;
    }
  }

}
