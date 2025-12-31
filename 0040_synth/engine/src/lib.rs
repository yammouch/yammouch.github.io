use wasm_bindgen::prelude::*;
use std::ops::{Add, AddAssign, Mul, MulAssign};

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  pub fn log(s: &str);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cplxpol {
  pub mag  : f64,
  pub angle: f64, // rad
}

impl Cplxpol {
  pub fn from_reim(re: f64, im: f64) -> Self {
    Self {
      mag  : (re*re+im*im).sqrt(),
      angle: im.atan2(re)
    }
  }

  pub fn re(&self) -> f64 {
    self.mag*self.angle.cos()
  }

  pub fn im(&self) -> f64 {
    self.mag*self.angle.sin()
  }
}

pub fn angle_regu(angle: f64) -> f64 {
  let pi = std::f64::consts::PI;
  angle - ((angle+pi)/(2.*pi)).floor()*(2.*pi)
}

impl Add for Cplxpol {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    let diff_angle = other.angle - self.angle;
    let mag = ( self.mag*self.mag + other.mag*other.mag
              + 2.*self.mag*other.mag*diff_angle.cos() )
            .max(0.).sqrt();
    let angle_add = (other.mag*diff_angle.sin()).atan2(
                     other.mag*diff_angle.cos()+self.mag );
    Self { mag: mag, angle: angle_regu(self.angle + angle_add) }
  }
}

impl Add<f64> for Cplxpol {
  type Output = Self;

  fn add(self, other: f64) -> Self {
    self + Cplxpol { mag: other, angle: 0. }
  }
}

impl AddAssign for Cplxpol {
  fn add_assign(&mut self, other: Self) {
    *self = self.clone() + other;
  }
}

impl AddAssign<f64> for Cplxpol {
  fn add_assign(&mut self, other: f64) {
    *self = self.clone() + other;
  }
}

impl Mul for Cplxpol {
  type Output = Self;

  fn mul(self, other: Self) -> Self {
    let mag = self.mag*other.mag;
    Self { mag: mag, angle: angle_regu(self.angle + other.angle) }
  }
}

impl MulAssign for Cplxpol {
  fn mul_assign(&mut self, other: Self) {
    *self = self.clone() * other;
  }
}

fn chord_root(pr1: &[bool]) -> Option<usize> {
  let mut oct = [0usize; 12];
  let mut low : Option<usize> = None;
  for i in 0..pr1.len() {
    if pr1[i] {
      if low == None {
        low = Some(i);
      }
      oct[ i    %12] += 5;
      oct[(i+ 5)%12] += 4;
      oct[(i+ 8)%12] += 3;
      oct[(i+10)%12] += 2;
      oct[(i+ 2)%12] += 1;
    }
  }
  let mx = oct.iter().enumerate().fold( (0usize, 0usize, 0usize),
   |(mx, imx, cnt), (i, &x)| {
    if x < mx {
      (mx, imx, cnt)
    } else if x == mx {
      (mx, imx, cnt+1)
    } else {
      (x, i, 1) 
    }
  });
  if mx.2 == 1 {
    Some(mx.1)
  } else {
    None
  }
}

fn tune(c: &mut [Cplxpol], n: isize, f: f64, t: &[f64]) {
  let i = (-n).div_euclid(t.len() as isize);
  let mut f0 = f*2f64.powf(i as f64);
  let mut j = (-n).rem_euclid(t.len() as isize) as usize;
  for k in 0..c.len() {
    c[k].angle = f0*t[j];
    j += 1;
    if t.len() <= j {
      j = 0;
      f0 *= 2.;
    }
  }
}

const JUST_TABLE : [f64; 12] = [
  1.0,
 17.0/16.0,
  9.0/ 8.0,
  6.0/ 5.0, //  7.0/ 6.0
  5.0/ 4.0,
  4.0/ 3.0,
 45.0/32.0, // 11.0/ 8.0
  3.0/ 2.0,
 25.0/16.0, //  8.0/ 5.0
 27.0/16.0, //  5.0/ 3.0
  9.0/ 5.0, //  7.0/ 4.0
 15.0/ 8.0,
];

#[derive(Debug)]
#[wasm_bindgen]
pub struct Source {
  v  : Vec<f32>,
  mst: f64,
  exc: Exc,
  rsn: Rsn,
  stt: Vec<Cplxpol>,
  eqt: Vec<f64>,
  crt: usize,
}

#[wasm_bindgen]
impl Source {
  pub fn new(f_master_a: f64) -> Self {
    let nk = 40;   // the number of keys
    let cfg = [0, 12, 19, 24, 28, 31, 36, 38]; // harmonicses
    let mx = cfg.into_iter().max().expect("empty array");
    let mut slf = Self {
      v: Vec::with_capacity(128),
      mst: f_master_a,
      exc: Exc::new(nk, &cfg),
      rsn: Rsn::new(nk, &cfg),
      stt: vec![Cplxpol{ mag: 0.0, angle: 0.0 }; nk+mx],
      eqt: (0..12).map( |i| 2f64.powf((i as f64)/12.)).collect(),
      crt: 0,
    };
    let tau = std::f64::consts::TAU;
    tune(&mut slf.rsn.c, 33, tau * f_master_a, &slf.eqt);
    slf
  }

  pub fn off(&mut self, i: usize) {
    self.rsn.off(i);
  }

  pub fn on(&mut self, i: usize) {
    let tau = std::f64::consts::TAU;
    let pi  = std::f64::consts::PI;
    self.rsn.on(i);
    self.exc.on(i);
    if let Some(k) = chord_root(&self.rsn.pr1) {
      self.crt = k;
      tune(&mut self.rsn.c, 21+k as isize, pi * self.mst * self.eqt[k], &JUST_TABLE);
    } else {
      self.crt = 12;
      tune(&mut self.rsn.c, 33, tau * self.mst, &self.eqt);
    }
  }

  pub fn tick(&mut self, n: usize) {
    self.v.clear();
    for _ in 0..n {
      self.exc.tick(&mut self.stt);
      self.rsn.tick(&mut self.stt);
      let s : f64 = self.stt.iter().map(Cplxpol::re).sum();
      self.v.push(s as f32);
    }
  }

  pub fn ptr(&self) -> *const f32 {
    self.v.as_ptr()
  }

  pub fn crt(&self) -> usize {
    self.crt
  }

  pub fn harm(&mut self, h: usize, mag: f64) {
    self.exc.harm(h, mag);
  }
}

fn k2r(nk: usize, cfg: &[usize]) -> Vec<Vec<(usize, usize)>> {
  let mx : usize = cfg.iter().max().expect("empty array").clone();
  let mut cnt : Vec<usize> = vec![0; nk+mx];
  let mut ret : Vec<Vec<(usize, usize)>> = vec![vec![]; nk];
  for c in cfg {
    for i in 0..nk {
      ret[i].push((c+i, cnt[c+i]));
      cnt[c+i] += 1;
    }
  }
  ret
}

#[derive(Debug, Clone, PartialEq)]
struct Rsn {
  c  : Vec<Cplxpol>,
  lim: Vec<f64>,
  dcn: Vec<f64>,
  dcf: Vec<f64>,
  k2r: Vec<Vec<(usize, usize)>>,
  prs: Vec<Vec<bool>>,
  pr1: Vec<bool>,
}

impl Rsn {
  fn new(nk: usize, cfg: &[usize]) -> Self {
    let mx = cfg.iter().max().expect("empty array").clone();
    let mut prs = vec![vec![]; nk+mx];
    for &c in cfg {
      for i in 0..nk {
        prs[c+i].push(false);
      }
    }
    Self {
      c  : vec![Cplxpol { mag: 1. - 1e-1, angle: 0.0 }; nk+mx],
      lim: vec![10.0; nk+mx],
      dcn: vec![1. - 1e-4; nk+mx],
      dcf: vec![1. - 1e-1; nk+mx],
      k2r: k2r(nk, cfg),
      prs,
      pr1: vec![false; nk],
    }
  }
  fn tick(&self, dst: &mut [Cplxpol]) {
    for i in 0..dst.len() {
      dst[i] *= self.c[i];
      dst[i].mag = dst[i].mag.min(self.lim[i]);
    }
  }
  fn on(&mut self, i: usize) {
    for &t in self.k2r[i].iter() {
      self.prs[t.0][t.1] = true;
      self.c[t.0].mag = self.dcn[t.0];
    }
    self.pr1[i] = true;
  }
  fn off(&mut self, i: usize) {
    for &t in self.k2r[i].iter() {
      self.prs[t.0][t.1] = false;
      if self.prs[t.0].iter().all( |&x| x == false ) {
        self.c[t.0].mag = self.dcf[t.0];
      }
    }
    self.pr1[i] = false;
  }
}

#[derive(Debug, Clone, PartialEq)]
struct Exc1 {
  n: Vec<usize>,
  v: Vec<Vec<Cplxpol>>,
}

impl Iterator for Exc1 {
  type Item = Cplxpol;

  fn next(&mut self) -> Option<Self::Item> {
    let mut acc = Cplxpol { mag: 0.0, angle: 0.0 };
    for i in 0..self.n.len() {
      if 0 < self.n[i] {
        self.n[i] -= 1;
        acc += self.v[i][self.n[i]]
      }
    }
    Some(acc)
  }
}

#[derive(Debug, Clone, PartialEq)]
struct Exc {
  a  : Vec<Exc1>,
  exi: Vec<Vec<(usize, usize)>>,
}

impl Exc {
  fn new(nk: usize, cfg: &[usize]) -> Self {
    let mx = cfg.iter().max().expect("empty array").clone();
    let mut a = vec![ Exc1 { n: vec![], v: vec![] }; nk+mx ];
    for &c in cfg {
      for i in 0..nk {
        a[c+i].n.push(0);
        a[c+i].v.push(vec![Cplxpol {mag: 1.0, angle: 0.0}]);
      }
    }
    Self { a,
           exi: k2r(nk, cfg) }
  }
  fn tick(&mut self, dst: &mut [Cplxpol]) {
    for i in 0..dst.len() {
      let c = self.a[i].next().expect("Exc1 no value");
      if c.mag != 0.0 {
        dst[i] += c;
      }
    }
  }
  fn on(&mut self, i: usize) {
    for &t in self.exi[i].iter() {
      self.a[t.0].n[t.1] = self.a[t.0].v[t.1].len();
    }
  }
  fn harm(&mut self, i: usize, mag: f64) {
    for v in &self.exi {
      let (i0, i1) = v[i];
      self.a[i0].v[i1][0].mag = mag;
    }
  }
}

#[cfg(test)]
mod cplxpol_test {
  use wasm_bindgen_test::*;
  use super::Cplxpol;

  fn points() -> Vec<(f64, f64)> {
    use std::iter::once;
    let crd = vec![3f64.sqrt()*0.5, 1.0, 3f64.sqrt()];
    let crd = crd.iter().rev().map(|&x| -x).chain(once(0f64))
              .chain(crd.iter().map(|&x| x)).collect::<Vec<_>>();
    let mut points : Vec<(f64, f64)> = vec![];
    for &re in &crd {
      for &im in &crd {
        points.push((re, im));
      }
    }
    points
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn add () {
    let pi = std::f64::consts::PI;
    let points = points();
    for &(are, aim) in &points {
      for &(bre, bim) in &points {
        let a = Cplxpol::from_reim(are, aim);
        let b = Cplxpol::from_reim(bre, bim);
        let sum = a + b;
        assert!((are + bre - sum.re()).abs() < 1e-6,
         "are: {are}, bre: {bre}, result: {}", sum.re());
        assert!((aim + bim - sum.im()).abs() < 1e-6,
         "aim: {aim}, bim: {bim}, result: {}", sum.im());
        assert!(sum.angle.abs() < pi+0.1,
         "angle: {}", sum.angle);
      }
    }
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn mul () {
    let pi = std::f64::consts::PI;
    let points = points();
    for &(are, aim) in &points {
      for &(bre, bim) in &points {
        let a = Cplxpol::from_reim(are, aim);
        let b = Cplxpol::from_reim(bre, bim);
        let prod = a * b;
        assert!((are*bre - aim*bim - prod.re()).abs() < 1e-6,
         "rere: {}, imim: {}, re: {}", are*bre, aim*bim, prod.re());
        assert!((aim*bre + are*bim - prod.im()).abs() < 1e-6,
         "imre: {}, reim: {}, im: {}", aim*bre, are*bim, prod.im());
        assert!(prod.angle.abs() < pi+0.1,
         "angle: {}", prod.angle);
      }
    }
  }
}

#[cfg(test)]
mod test_vecreson {
  use wasm_bindgen_test::*;

  #[wasm_bindgen_test(unsupported = test)]
  fn test_tune() {
    use super::Cplxpol;
    use super::tune;
    let mut c = vec![ Cplxpol { mag: 1.0, angle: 0.0 },
                      Cplxpol { mag: 1.0, angle: 0.0 },
                      Cplxpol { mag: 1.0, angle: 0.0 },
                      Cplxpol { mag: 1.0, angle: 0.0 } ];
    tune(&mut c, 0, 2.0, &[1.0, 1.5]);
    assert_eq!(c,
     vec![ Cplxpol { mag: 1.0, angle: 2.0 },
           Cplxpol { mag: 1.0, angle: 3.0 },
           Cplxpol { mag: 1.0, angle: 4.0 },
           Cplxpol { mag: 1.0, angle: 6.0 } ]);
    tune(&mut c, 1, 2.0, &[1.0, 1.5]);
    assert_eq!(c,
     vec![ Cplxpol { mag: 1.0, angle: 1.5 },
           Cplxpol { mag: 1.0, angle: 2.0 },
           Cplxpol { mag: 1.0, angle: 3.0 },
           Cplxpol { mag: 1.0, angle: 4.0 } ]);
    tune(&mut c, 2, 2.0, &[1.0, 1.5]);
    assert_eq!(c,
     vec![ Cplxpol { mag: 1.0, angle: 1.0 },
           Cplxpol { mag: 1.0, angle: 1.5 },
           Cplxpol { mag: 1.0, angle: 2.0 },
           Cplxpol { mag: 1.0, angle: 3.0 } ]);
    tune(&mut c, 3, 2.0, &[1.0, 1.5]);
    assert_eq!(c,
     vec![ Cplxpol { mag: 1.0, angle: 0.75 },
           Cplxpol { mag: 1.0, angle: 1.0  },
           Cplxpol { mag: 1.0, angle: 1.5  },
           Cplxpol { mag: 1.0, angle: 2.0  } ]);
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn test_chord_root() {
    use super::chord_root;
    assert_eq!(Some(0), chord_root(&[
     true, // C
     false, false, false,
     true, // E
     false, false,
     true, // G
     false, false, false, false]));
    assert_eq!(Some(5), chord_root(&[
     true, // C
     false, false, false, false,
     true, // F
     false, false, false,
     true, // A
     false, false]));
    assert_eq!(Some(4), chord_root(&[
     false, false, false, false,
     true, // E
     false, false,
     true, // G
     false, false, false,
     true ])); // B
    assert_eq!(Some(7), chord_root(&[
     false, false,
     true, // D
     false,
     true, // E
     false, false,
     true, // G
     false, false, false,
     true ])); // B
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn test_k2r() {
    let rv = super::k2r(13, &[0]);
    assert_eq!(rv,
     vec![vec![(0, 0)], vec![(1, 0)], vec![(2, 0)], vec![(3, 0)], vec![(4, 0)],
          vec![(5, 0)], vec![(6, 0)], vec![(7, 0)], vec![(8, 0)], vec![(9, 0)],
          vec![(10, 0)], vec![(11, 0)], vec![(12, 0)]]);

    let rv = super::k2r(13, &[0, 12]);
    assert_eq!(rv,
     vec![[(0, 0), (12, 1)],
          [(1, 0), (13, 0)],
          [(2, 0), (14, 0)],
          [(3, 0), (15, 0)],
          [(4, 0), (16, 0)],
          [(5, 0), (17, 0)],
          [(6, 0), (18, 0)],
          [(7, 0), (19, 0)],
          [(8, 0), (20, 0)],
          [(9, 0), (21, 0)],
          [(10, 0), (22, 0)],
          [(11, 0), (23, 0)],
          [(12, 0), (24, 0)]]);
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn test_rsn() {
    use super::Cplxpol;
    use super::Rsn;

    let mut rsn = Rsn {
      c  : vec![Cplxpol { mag: 0.25, angle: 0.25 },
                Cplxpol { mag: 0.75, angle: 0.5  }],
      lim: vec![1.0 , 1.0 ],
      dcn: vec![0.5 , 1.5 ],
      dcf: vec![0.25, 0.75],
      k2r: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
      prs: vec![vec![false],
                vec![false, false]],
      pr1: vec![false, false],
    };

    rsn.on(0);
    assert_eq!(rsn, Rsn {
      c  : vec![Cplxpol { mag: 0.5, angle: 0.25 },
                Cplxpol { mag: 1.5, angle: 0.5  }],
      lim: vec![1.0 , 1.0 ],
      dcn: vec![0.5 , 1.5 ],
      dcf: vec![0.25, 0.75],
      k2r: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
      prs: vec![vec![true],
                vec![false, true]],
      pr1: vec![true, false],
    });

    let mut v = vec![Cplxpol { mag: 1.0, angle: 0.0 },
                     Cplxpol { mag: 1.0, angle: 0.0 }];
    rsn.tick(&mut v[..]);
    assert_eq!(v, vec![
     Cplxpol { mag: 0.5, angle: 0.25 },
     Cplxpol { mag: 1.0, angle: 0.5  },
    ]);

    rsn.on(1);
    assert_eq!(rsn, Rsn {
      c  : vec![Cplxpol { mag: 0.5, angle: 0.25 },
                Cplxpol { mag: 1.5, angle: 0.5  }],
      lim: vec![1.0 , 1.0 ],
      dcn: vec![0.5 , 1.5 ],
      dcf: vec![0.25, 0.75],
      k2r: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
      prs: vec![vec![true],
                vec![true, true]],
      pr1: vec![true, true],
    });

    rsn.off(0);
    assert_eq!(rsn, Rsn {
      c  : vec![Cplxpol { mag: 0.25, angle: 0.25 },
                Cplxpol { mag: 1.5 , angle: 0.5  }],
      lim: vec![1.0 , 1.0 ],
      dcn: vec![0.5 , 1.5 ],
      dcf: vec![0.25, 0.75],
      k2r: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
      prs: vec![vec![false],
                vec![true, false]],
      pr1: vec![false, true],
    });

    rsn.off(1);
    assert_eq!(rsn, Rsn {
      c  : vec![Cplxpol { mag: 0.25, angle: 0.25 },
                Cplxpol { mag: 0.75, angle: 0.5  }],
      lim: vec![1.0 , 1.0 ],
      dcn: vec![0.5 , 1.5 ],
      dcf: vec![0.25, 0.75],
      k2r: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
      prs: vec![vec![false],
                vec![false, false]],
      pr1: vec![false, false],
    });
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn test_exc1() {
    use super::Cplxpol;
    use super::Exc1;

    let mut exc1 = Exc1 {
      n: vec![1usize, 2],
      v: vec![vec![ Cplxpol { mag: 1.0, angle: 1.0 } ],
              vec![ Cplxpol { mag: 1.0, angle: 0.0 },
                    Cplxpol { mag: 0.5, angle: 0.0 } ]],
    };

    let e = exc1.next().unwrap();
    assert_eq!(e, Cplxpol { mag: 1.0, angle: 1.0 } +
                  Cplxpol { mag: 0.5, angle: 0.0 } );
    assert_eq!(exc1, Exc1 {
      n: vec![0usize, 1],
      v: vec![vec![ Cplxpol { mag: 1.0, angle: 1.0 } ],
              vec![ Cplxpol { mag: 1.0, angle: 0.0 },
                    Cplxpol { mag: 0.5, angle: 0.0 } ]],
    });

    let e = exc1.next().unwrap();
    assert_eq!(e, Cplxpol { mag: 1.0, angle: 0.0 });
    assert_eq!(exc1, Exc1 {
      n: vec![0usize, 0],
      v: vec![vec![ Cplxpol { mag: 1.0, angle: 1.0 } ],
              vec![ Cplxpol { mag: 1.0, angle: 0.0 },
                    Cplxpol { mag: 0.5, angle: 0.0 } ]],
    });
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn test_exc() {
    use super::Cplxpol;
    use super::Exc1;
    use super::Exc;
    let mut exc = Exc {
      a  : vec![
        Exc1 {
          n: vec![0],
          v: vec![vec![ Cplxpol { mag: 1.0, angle: 0.0 } ]],
        },
        Exc1 {
          n: vec![0, 0],
          v: vec![vec![ Cplxpol { mag: 1.0, angle: 0.0 } ],
                  vec![ Cplxpol { mag: 0.5, angle: 0.0 } ] ],
        },
      ],
      exi: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
    };
    exc.on(0);
    assert_eq!(exc, Exc {
      a  : vec![
        Exc1 {
          n: vec![1],
          v: vec![vec![ Cplxpol { mag: 1.0, angle: 0.0 } ]],
        },
        Exc1 {
          n: vec![0, 1],
          v: vec![vec![ Cplxpol { mag: 1.0, angle: 0.0 } ],
                  vec![ Cplxpol { mag: 0.5, angle: 0.0 } ] ],
        },
      ],
      exi: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
    });
    let mut o = vec![Cplxpol { mag: 0.0, angle: 0.0 },
                     Cplxpol { mag: 1.0, angle: 0.0 }];
    exc.tick(&mut o);
    assert_eq!(o,
     vec![Cplxpol { mag: 1.0, angle: 0.0 },
          Cplxpol { mag: 1.5, angle: 0.0 }] );
    assert_eq!(exc, Exc {
      a  : vec![
        Exc1 {
          n: vec![0],
          v: vec![vec![ Cplxpol { mag: 1.0, angle: 0.0 } ]],
        },
        Exc1 {
          n: vec![0, 0],
          v: vec![vec![ Cplxpol { mag: 1.0, angle: 0.0 } ],
                  vec![ Cplxpol { mag: 0.5, angle: 0.0 } ] ],
        },
      ],
      exi: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
    });
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn test_exc_harm() {
    use super::Cplxpol;
    use super::Exc1;
    use super::Exc;
    let mut exc = Exc {
      a  : vec![
        Exc1 {
          n: vec![0],
          v: vec![vec![ Cplxpol { mag: 1.0, angle: 0.0 } ]],
        },
        Exc1 {
          n: vec![0, 0],
          v: vec![vec![ Cplxpol { mag: 1.0, angle: 0.0 } ],
                  vec![ Cplxpol { mag: 0.5, angle: 0.0 } ] ],
        },
      ],
      exi: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
    };
    exc.harm(0, 0.75);
    assert_eq!(exc, Exc {
      a  : vec![
        Exc1 {
          n: vec![0],
          v: vec![vec![ Cplxpol { mag: 0.75, angle: 0.0 } ]],
        },
        Exc1 {
          n: vec![0, 0],
          v: vec![vec![ Cplxpol { mag: 0.75, angle: 0.0 } ],
                  vec![ Cplxpol { mag: 0.5 , angle: 0.0 } ] ],
        },
      ],
      exi: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
    });
  }
}
