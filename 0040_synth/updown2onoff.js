export function updown2note_build(notekey) {
  let o = {};
  notekey.forEach( (a, i) => {
    a.forEach( (k) => {
      o[k] = i;
    })
  });
  return o;
}

export function note2onoff_build(notekey) {
  return notekey.map( (a) => {
    let o = {};
    a.forEach( (x) => {
      o[x] = false;
    });
    return o;
  });
}

let notekey = [
  ["Digit1"],
  [],
  ["KeyQ"],
  ["ShiftLeft", "Digit2"],
  ["KeyA"],
  ["KeyW"],
  ["KeyZ", "Digit3"],
  ["KeyS"],
  ["KeyE"],
  ["KeyX", "Digit4"],
  ["KeyD"],
  ["KeyR"],
  ["KeyC", "Digit5"],
  ["KeyF"],
  ["KeyT"],
  ["KeyV", "Digit6"],
  ["KeyG"],
  ["KeyY"],
  ["KeyB", "Digit7"],
  ["KeyH"],
  ["KeyU"],
  ["KeyN", "Digit8"],
  ["KeyJ"],
  ["KeyI"],
  ["KeyM", "Digit9"],
  ["KeyK"],
  ["KeyO"],
  ["Comma", "Digit0"],
  ["KeyL"],
  ["KeyP"],
  ["Period", "Minus"],
  ["Semicolon"],
  ["BracketLeft"],
  ["Slash", "Equal"],
  ["Quote"],
  ["BracketRight"],
  ["IntlRo", "IntlYen"],
  ["Backslash"],
  ["Enter"],
  ["ShiftRight"]
];

export class UpDown2OnOff {

  constructor() {
    this.note2onoff = note2onoff_build(notekey);
    this.updown2note = updown2note_build(notekey);
  }

  on(key) {
    const note = this.updown2note[key];
    console.log(note);
    if (note === undefined) { return; }
    console.log(this.note2onoff[note]);
    if (this.note2onoff[note][key]) { return; }
    this.note2onoff[note][key] = true;
    return { cmd: "on", note: note };
  }

  off(key) {
    const note = this.updown2note[key];
    if (note === undefined) { return; }
    this.note2onoff[note][key] = false;
    if (Object.entries(this.note2onoff[note]).some((k) => k[1])) { return; }
    return { cmd: "off", note: note };
  }

}
