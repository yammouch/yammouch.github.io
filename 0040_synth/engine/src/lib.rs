use wasm_bindgen::prelude::*;
use std::ops::{Add, AddAssign, Mul, MulAssign, Rem, RemAssign};
use std::marker::PhantomData;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  pub fn log(s: &str);
}

type Flt = f32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cplxpol {
  pub mag  : Flt,
  pub angle: Flt, // rad
}

trait Cplx
 : Mul<Output=Self>
 + MulAssign
 + Add<Output=Self>
 + AddAssign
 + Sized
 + Clone
 + Copy
 + From<Flt>
{
  fn from_magangle(mag: Flt, angle: Flt) -> Self;
  fn lim(&mut self, l: Flt);
}

impl Cplxpol {
  pub fn from_reim(re: Flt, im: Flt) -> Self {
    Self {
      mag  : (re*re+im*im).sqrt(),
      angle: im.atan2(re)
    }
  }

  pub fn re(&self) -> Flt {
    self.mag*self.angle.cos()
  }

  pub fn im(&self) -> Flt {
    self.mag*self.angle.sin()
  }
}

impl Cplx for Cplxpol {
  fn from_magangle(mag: Flt, angle: Flt) -> Self {
    Self {
      mag,
      angle,
    }
  }

  fn lim(&mut self, l: Flt) {
    if l < self.mag {
      self.mag = l;
    }
  }
}

pub fn angle_regu(angle: Flt) -> Flt {
  let pi = std::f64::consts::PI as Flt;
  angle - ((angle+pi)/(2.*pi)).floor()*(2.*pi)
}

impl From<Flt> for Cplxpol {
  fn from(x: Flt) -> Self {
    Self { mag: x, angle: 0.0 }
  }
}

impl<T> Add<T> for Cplxpol where
 T: Into<Cplxpol> {
  type Output = Self;

  fn add(self, other: T) -> Self {
    let other = other.into();
    let diff_angle = other.angle - self.angle;
    let mag = ( self.mag*self.mag + other.mag*other.mag
              + 2.*self.mag*other.mag*diff_angle.cos() )
            .max(0.).sqrt();
    let angle_add = (other.mag*diff_angle.sin()).atan2(
                     other.mag*diff_angle.cos()+self.mag );
    Self { mag: mag, angle: angle_regu(self.angle + angle_add) }
  }
}

impl<T> AddAssign<T> for Cplxpol where
 T: Into<Cplxpol> {
  fn add_assign(&mut self, other: T) {
    *self = self.clone() + other.into();
  }
}

impl<T> Mul<T> for Cplxpol where
 T: Into<Cplxpol> {
  type Output = Self;

  fn mul(self, other: T) -> Self {
    let other = other.into();
    let mag = self.mag*other.mag;
    Self { mag: mag, angle: angle_regu(self.angle + other.angle) }
  }
}

impl<T> MulAssign<T> for Cplxpol where
 T: Into<Cplxpol> {
  fn mul_assign(&mut self, other: T) {
    let other = other.into();
    *self = self.clone() * other;
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cplxy {
  pub x : Flt,
  pub y : Flt,
}

impl Cplxy {
  pub fn from_reim(re: Flt, im: Flt) -> Self {
    Self {
      x: re,
      y: im,
    }
  }

  pub fn re(&self) -> Flt {
    self.x
  }

  pub fn im(&self) -> Flt {
    self.y
  }
}

impl Cplx for Cplxy {
  fn from_magangle(mag: Flt, angle: Flt) -> Self {
    Self {
      x : mag*angle.cos(),
      y : mag*angle.sin(),
    }
  }

  fn lim(&mut self, l: Flt) {
    let mag2 = self.x*self.x + self.y*self.y;
    let l2 = l*l;
    if l2 < mag2 {
      let r = (l2/mag2).sqrt();
      self.x *= r;
      self.y *= r;
    }
  }
}

impl From<Flt> for Cplxy {
  fn from(x: Flt) -> Self {
    Self { x, y: 0.0 }
  }
}

impl<T> Add<T> for Cplxy where
 T: Into<Cplxy> {
  type Output = Self;

  fn add(self, other: T) -> Self {
    let other = other.into();
    Self { x: self.x + other.x, y: self.y + other.y }
  }
}

impl<T> AddAssign<T> for Cplxy where
 T: Into<Cplxy> {
  fn add_assign(&mut self, other: T) {
    *self = self.clone() + other.into();
  }
}

impl<T> Mul<T> for Cplxy where
 T: Into<Cplxy> {
  type Output = Self;

  fn mul(self, other: T) -> Self {
    let other = other.into();
    Self {
      x: self.x*other.x - self.y*other.y,
      y: self.x*other.y + self.y*other.x,
    }
  }
}

impl<T> MulAssign<T> for Cplxy where
 T: Into<Cplxy> {
  fn mul_assign(&mut self, other: T) {
    let other = other.into();
    *self = self.clone() * other;
  }
}

fn gcd2<T>(mut a: T, mut b: T) -> T where
  T: PartialEq + PartialOrd + Rem<Output = T> + RemAssign + From<u8> + Copy,
{
  if a <= b {
    if b % a == 0u8.into() {
      return a
    }
    b %= a;
  }
  loop {
    if b == 0u8.into() {
      return a;
    }
    a %= b;
    if a == 0u8.into() {
      return b;
    }
    b %= a;
  }
}

fn chord_root2(pr1: &[bool], den: &[u32]) {
  for i in 0..den.len() {
    let a = pr1.iter().cycle().skip(i).zip(den).filter_map( |(&b, &x)| {
      if b { Some(x) } else { None }
    });
    println!("{:?}", a.collect::<Vec<_>>());
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

fn tune<C: Cplx>(c: &mut [C], n: isize, f: Flt, t: &[Flt]) {
  let i = (-n).div_euclid(t.len() as isize);
  let mut f0 = f*(2 as Flt).powf(i as Flt);
  let mut j = (-n).rem_euclid(t.len() as isize) as usize;
  for k in 0..c.len() {
    c[k] = C::from_magangle(1., f0*t[j]);
    j += 1;
    if t.len() <= j {
      j = 0;
      f0 *= 2.;
    }
  }
}

const JUST_TABLE : [Flt; 12] = [
  1.0,
 17.0/16.0,
  9.0/ 8.0,
  6.0/ 5.0, //  7.0/ 6.0
  5.0/ 4.0,
 27.0/20.0, //  4.0/ 3.0
 45.0/32.0, // 11.0/ 8.0
  3.0/ 2.0,
 25.0/16.0, //  8.0/ 5.0
 27.0/16.0, //  5.0/ 3.0
  9.0/ 5.0, //  7.0/ 4.0
 15.0/ 8.0,
];

type Cpl = Cplxy;

#[derive(Debug)]
#[wasm_bindgen]
pub struct Source {
  v  : Vec<f32>,
  exc: Exc<Cpl>,
  rsn: Rsn<Cpl>,
  stt: Vec<Cpl>,
}

#[wasm_bindgen]
impl Source {
  pub fn new(f_master_a: Flt) -> Self {
    let nk = 40;   // the number of keys
    let cfg = [0, 12, 19, 24, 28, 31, 36, 38]; // harmonicses
    let mx = cfg.into_iter().max().expect("empty array");
    let slf = Self {
      v: Vec::with_capacity(128),
      exc: Exc::new(nk, &cfg, f_master_a),
      rsn: Rsn::new(nk, &cfg, f_master_a),
      stt: vec![Cpl::from_magangle(0.0, 0.0); nk+mx],
    };
    slf
  }

  pub fn off(&mut self, i: usize) {
    self.rsn.off(i);
  }

  pub fn on(&mut self, i: usize) {
    self.rsn.on(i);
    self.exc.on(i);
  }

  pub fn tick(&mut self, n: usize) {
    self.v.clear();
    for _ in 0..n {
      self.exc.tick(&mut self.stt);
      self.rsn.tick(&mut self.stt);
      let s : Flt = self.stt.iter().map(Cpl::re).sum();
      self.v.push(s as f32);
    }
  }

  pub fn ptr(&self) -> *const f32 {
    self.v.as_ptr()
  }

  pub fn crt(&self) -> usize {
    self.rsn.crt
  }

  pub fn harm(&mut self, h: usize, mag: Flt) {
    self.exc.harm(h, mag);
    self.rsn.harm(h, mag);
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
struct Rsn<C> {
  c  : Vec<C>,
  lim: Vec<Flt>,
  atv: Vec<bool>,
  dcn: Vec<Flt>,
  dcf: Vec<Flt>,
  k2r: Vec<Vec<(usize, usize)>>,
  prs: Vec<Vec<bool>>,
  pr1: Vec<bool>,
  drb: [Flt; 8],
  crt: usize,
  ftb: Vec<Vec<C>>,
}

impl<C: Cplx> Rsn<C> {
  fn new(nk: usize, cfg: &[usize], f_master_a: Flt) -> Self {
    let pi = std::f64::consts::PI as Flt;
    let tau = std::f64::consts::TAU as Flt;
    let mx = cfg.iter().max().expect("empty array").clone();
    let mut prs = vec![vec![]; nk+mx];
    for &c in cfg {
      for i in 0..nk {
        prs[c+i].push(false);
      }
    }
    let mut ftb = vec![vec![1.0.into(); nk+mx]; 13];
    let eqt = (0..12).map( |i|
     (2 as Flt).powf((i as Flt)/12.)
    ).collect::<Vec<_>>();
    tune(&mut ftb[12], 33, tau * f_master_a, &eqt);
    for i in 0..12 {
      tune(
       &mut ftb[i],
       24+i as isize,
       pi * f_master_a * (2 as Flt).powf(((i+3) as Flt)/12.),
       &JUST_TABLE);
    }
    Self {
      c  : vec![C::from_magangle(1. - 1e-2, 0.0); nk+mx],
      lim: vec![10.0; nk+mx],
      atv: vec![false; nk+mx],
      dcn: vec![1. - 1e-4; nk+mx],
      dcf: vec![1. - 1e-2; nk+mx],
      k2r: k2r(nk, cfg),
      prs,
      pr1: vec![false; nk],
      drb: [1.0; 8],
      crt: 12,
      ftb,
    }
  }
  fn tick(&self, dst: &mut [C]) {
    for i in 0..dst.len() {
      dst[i] *= self.c[i];
      dst[i].lim(self.lim[i]);
    }
  }
  fn on(&mut self, i: usize) {
    for (&t, d) in self.k2r[i].iter().zip(self.drb) {
      if d != 0.0 {
        self.prs[t.0][t.1] = true;
        self.atv[t.0] = true;
      }
    }
    self.pr1[i] = true;
    if let Some(k) = chord_root(&self.pr1) {
      self.crt = k;
    } else {
      self.crt = 12;
    }
    for i in 0..self.c.len() {
      let dcy = if self.atv[i] { self.dcn[i] } else { self.dcf[i] };
      self.c[i] = self.ftb[self.crt][i] * dcy.into();
    }
  }
  fn off(&mut self, i: usize) {
    for &t in self.k2r[i].iter() {
      self.prs[t.0][t.1] = false;
      if self.prs[t.0].iter().all( |&x| x == false ) {
        self.c[t.0] = self.ftb[self.crt][t.0] * self.dcf[t.0].into();
        self.atv[t.0] = false;
      }
    }
    self.pr1[i] = false;
  }
  fn harm(&mut self, h: usize, mag: Flt) {
    self.drb[h] = mag;
    if mag == 0.0 {
      for &t in self.k2r.iter().filter_map( |s| s.get(h) ) {
        self.prs[t.0][t.1] = false;
        if self.prs[t.0].iter().all( |&x| x == false ) {
          self.c[t.0] = self.ftb[self.crt][t.0] * self.dcf[t.0].into();
          self.atv[t.0] = false;
        }
      }
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
struct Exc1 {
  n: usize,
  v: Vec<Flt>,
  a: Flt,
}

impl Iterator for Exc1 {
  type Item = Flt;

  fn next(&mut self) -> Option<Self::Item> {
    if 0 < self.n {
      self.n -= 1;
      Some(self.v[self.n] * self.a)
    } else {
      Some(0.0)
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
struct Exc<C> {
  a  : Vec<Vec<Exc1>>,
  exi: Vec<Vec<(usize, usize)>>,
  _p : PhantomData<C>,
}

impl<C: Cplx> Exc<C> {
  fn new(nk: usize, cfg: &[usize], f_master_a: Flt) -> Self {
    let mx = cfg.iter().max().expect("empty array").clone();
    let mut a = vec![ Vec::<Exc1>::new(); nk+mx ];
    for &c in cfg {
      for i in 0..nk {
        let f = f_master_a*(2 as Flt).powf(((c + i) as Flt - 33.0)/12.0);
        a[c+i].push( Exc1 {
          n: 0,
          v: vec![f/0.5; (0.5/f).round_ties_even() as usize],
          a: 1.0,
        });
      }
    }
    Self { a,
           exi: k2r(nk, cfg),
           _p : PhantomData }
  }
  fn tick(&mut self, dst: &mut [C]) {
    for i in 0..dst.len() {
      let mut c = 0.0;
      for e in &mut self.a[i] {
        c += e.next().unwrap();
      }
      if c != 0.0 {
        dst[i] += c.into();
      }
    }
  }
  fn on(&mut self, i: usize) {
    for &t in self.exi[i].iter() {
      self.a[t.0][t.1].n = self.a[t.0][t.1].v.len();
    }
  }
  fn harm(&mut self, i: usize, mag: Flt) {
    for v in &self.exi {
      let (i0, i1) = v[i];
      self.a[i0][i1].a = mag;
    }
  }
}

#[cfg(test)]
mod cplxpol_test {
  use wasm_bindgen_test::*;
  use super::Cplxpol;
  use super::Flt;

  fn points() -> Vec<(Flt, Flt)> {
    use std::iter::once;
    let crd = vec![(3 as Flt).sqrt()*0.5, 1.0, (3 as Flt).sqrt()];
    let crd = crd.iter().rev().map(|&x| -x).chain(once(0 as Flt))
              .chain(crd.iter().map(|&x| x)).collect::<Vec<_>>();
    let mut points : Vec<(Flt, Flt)> = vec![];
    for &re in &crd {
      for &im in &crd {
        points.push((re, im));
      }
    }
    points
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn add () {
    let pi = std::f64::consts::PI as Flt;
    let points = points();
    for &(are, aim) in &points {
      for &(bre, bim) in &points {
        let a = Cplxpol::from_reim(are, aim);
        let b = Cplxpol::from_reim(bre, bim);
        let sum = a + b;
        assert!((are + bre - sum.re()).abs() < 1e-5,
         "are: {are}, bre: {bre}, result: {}", sum.re());
        assert!((aim + bim - sum.im()).abs() < 1e-5,
         "aim: {aim}, bim: {bim}, result: {}", sum.im());
        assert!(sum.angle.abs() < pi+0.1,
         "angle: {}", sum.angle);
      }
    }
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn mul () {
    let pi = std::f64::consts::PI as Flt;
    let points = points();
    for &(are, aim) in &points {
      for &(bre, bim) in &points {
        let a = Cplxpol::from_reim(are, aim);
        let b = Cplxpol::from_reim(bre, bim);
        let prod = a * b;
        assert!((are*bre - aim*bim - prod.re()).abs() < 1e-5,
         "rere: {}, imim: {}, re: {}", are*bre, aim*bim, prod.re());
        assert!((aim*bre + are*bim - prod.im()).abs() < 1e-5,
         "imre: {}, reim: {}, im: {}", aim*bre, are*bim, prod.im());
        assert!(prod.angle.abs() < pi+0.1,
         "angle: {}", prod.angle);
      }
    }
  }
}

#[cfg(test)]
mod cplxy_test {
  use wasm_bindgen_test::*;
  use super::Cplxy;
  use super::Flt;

  fn points() -> Vec<(Flt, Flt)> {
    use std::iter::once;
    let crd = vec![(3 as Flt).sqrt()*0.5, 1.0, (3 as Flt).sqrt()];
    let crd = crd.iter().rev().map(|&x| -x).chain(once(0 as Flt))
              .chain(crd.iter().map(|&x| x)).collect::<Vec<_>>();
    let mut points : Vec<(Flt, Flt)> = vec![];
    for &re in &crd {
      for &im in &crd {
        points.push((re, im));
      }
    }
    points
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn add () {
    let points = points();
    for &(are, aim) in &points {
      for &(bre, bim) in &points {
        let a = Cplxy::from_reim(are, aim);
        let b = Cplxy::from_reim(bre, bim);
        let sum = a + b;
        assert!((are + bre - sum.re()).abs() < 1e-6,
         "are: {are}, bre: {bre}, result: {}", sum.re());
        assert!((aim + bim - sum.im()).abs() < 1e-6,
         "aim: {aim}, bim: {bim}, result: {}", sum.im());
        //assert!(sum.angle.abs() < pi+0.1,
        // "angle: {}", sum.angle);
      }
    }
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn mul () {
    let points = points();
    for &(are, aim) in &points {
      for &(bre, bim) in &points {
        let a = Cplxy::from_reim(are, aim);
        let b = Cplxy::from_reim(bre, bim);
        let prod = a * b;
        assert!((are*bre - aim*bim - prod.re()).abs() < 1e-6,
         "rere: {}, imim: {}, re: {}", are*bre, aim*bim, prod.re());
        assert!((aim*bre + are*bim - prod.im()).abs() < 1e-6,
         "imre: {}, reim: {}, im: {}", aim*bre, are*bim, prod.im());
        //assert!(prod.angle.abs() < pi+0.1,
        // "angle: {}", prod.angle);
      }
    }
  }
}

#[cfg(test)]
mod gcd_test {
  use wasm_bindgen_test::*;

  #[wasm_bindgen_test(unsupported = test)]
  fn test_gcd2() {
    use super::gcd2;
    assert_eq!(gcd2(6, 3), 3);
    assert_eq!(gcd2(3, 6), 3);
    assert_eq!(gcd2(12, 4), 4);
    assert_eq!(gcd2(4, 12), 4);
    assert_eq!(gcd2(7, 4), 1);
    assert_eq!(gcd2(4, 7), 1);
    assert_eq!(gcd2(8, 8), 8);
    assert_eq!(gcd2(1, 8), 1);
    assert_eq!(gcd2(8, 1), 1);
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn test_chord_root2() {
    use super::chord_root2;
    chord_root2(&[true , false, false, false, true , false,
                  false, true , false, false, false, false],
                &[  160,   170,   180,   192,   200,   216,
                    225,   240,   250,   270,   288,   300]);
    panic!();
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
      atv: vec![false; 2],
      dcn: vec![0.5 , 1.5 ],
      dcf: vec![0.25, 0.75],
      k2r: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
      prs: vec![vec![false],
                vec![false, false]],
      pr1: vec![false, false],
      drb: [1.0; 8],
      crt: 0,
      ftb: vec![vec![Cplxpol { mag: 1.0, angle: 0.25 },
                     Cplxpol { mag: 1.0, angle: 0.5  }] ; 13],
    };

    rsn.on(0);
    assert_eq!(rsn.c,
      vec![Cplxpol { mag: 0.5, angle: 0.25 },
           Cplxpol { mag: 1.5, angle: 0.5  }]);
    assert_eq!(rsn.prs,
      vec![vec![true],
           vec![false, true]]);
    assert_eq!(rsn.pr1, vec![true, false]);

    let mut v = vec![Cplxpol { mag: 1.0, angle: 0.0 },
                     Cplxpol { mag: 1.0, angle: 0.0 }];
    rsn.tick(&mut v[..]);
    assert_eq!(v, vec![
     Cplxpol { mag: 0.5, angle: 0.25 },
     Cplxpol { mag: 1.0, angle: 0.5  },
    ]);

    rsn.on(1);
    assert_eq!(rsn.c,
      vec![Cplxpol { mag: 0.5, angle: 0.25 },
           Cplxpol { mag: 1.5, angle: 0.5  }]);
    assert_eq!(rsn.prs,
      vec![vec![true],
           vec![true, true]]);
    assert_eq!(rsn.pr1, vec![true, true]);

    rsn.off(0);
    assert_eq!(rsn.c,
      vec![Cplxpol { mag: 0.25, angle: 0.25 },
           Cplxpol { mag: 1.5 , angle: 0.5  }]);
    assert_eq!(rsn.prs,
      vec![vec![false],
           vec![true, false]]);
    assert_eq!(rsn.pr1, vec![false, true]);

    rsn.off(1);
    assert_eq!(rsn.c,
      vec![Cplxpol { mag: 0.25, angle: 0.25 },
           Cplxpol { mag: 0.75, angle: 0.5  }]);
    assert_eq!(rsn.prs,
      vec![vec![false],
           vec![false, false]]);
    assert_eq!(rsn.pr1, vec![false, false]);
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn test_rsn_harm() {
    use super::Cplxpol;
    use super::Rsn;

    let mut rsn = Rsn {
      c  : vec![Cplxpol { mag: 0.25, angle: 0.25 },
                Cplxpol { mag: 0.75, angle: 0.5  }],
      lim: vec![1.0 , 1.0 ],
      atv: vec![false; 2],
      dcn: vec![0.5 , 1.5 ],
      dcf: vec![0.25, 0.75],
      k2r: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
      prs: vec![vec![false],
                vec![false, false]],
      pr1: vec![false, false],
      drb: [1.0; 8],
      crt: 0,
      ftb: vec![vec![Cplxpol { mag: 1.0, angle: 0.25 },
                     Cplxpol { mag: 1.0, angle: 0.5  }]; 13],
    };

    rsn.on(0);
    assert_eq!(rsn.c,
      vec![Cplxpol { mag: 0.5, angle: 0.25 },
           Cplxpol { mag: 1.5, angle: 0.5  }]);
    assert_eq!(rsn.prs,
      vec![vec![true],
           vec![false, true]]);
    assert_eq!(rsn.pr1, vec![true, false]);

    rsn.harm(1, 0.0);
    assert_eq!(rsn.c,
      vec![Cplxpol { mag: 0.5 , angle: 0.25 },
           Cplxpol { mag: 0.75, angle: 0.5  }]);
    assert_eq!(rsn.prs,
      vec![vec![true],
           vec![false, false]]);
    assert_eq!(rsn.pr1, vec![true, false]);
    assert_eq!(rsn.drb, [1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);

    rsn.on(0);
    assert_eq!(rsn.c,
      vec![Cplxpol { mag: 0.5 , angle: 0.25 },
           Cplxpol { mag: 0.75, angle: 0.5  }]);
    assert_eq!(rsn.prs,
      vec![vec![true],
           vec![false, false]]);
    assert_eq!(rsn.pr1, vec![true, false]);
    assert_eq!(rsn.drb, [1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn test_exc1() {
    use super::Exc1;

    let mut exc1 = Exc1 {
      n: 2,
      v: vec![1.0, 0.5],
      a: 1.0,
    };

    let e = exc1.next().unwrap();
    assert_eq!(e, 0.5);
    assert_eq!(exc1, Exc1 {
      n: 1,
      v: vec![1.0, 0.5],
      a: 1.0,
    });

    let e = exc1.next().unwrap();
    assert_eq!(e, 1.0);
    assert_eq!(exc1, Exc1 {
      n: 0,
      v: vec![1.0, 0.5],
      a: 1.0,
    });
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn test_exc() {
    use super::Cplxpol;
    use super::Exc1;
    use super::Exc;
    use std::marker::PhantomData;
    let mut exc = Exc {
      a  : vec![
        vec![
          Exc1 {
            n: 0,
            v: vec![1.0],
            a: 1.0,
          }],
        vec![
          Exc1 {
            n: 0,
            v: vec![1.0],
            a: 1.0,
          },
          Exc1 {
            n: 0,
            v: vec![0.5],
            a: 1.0,
          }],
      ],
      exi: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
      _p : PhantomData,
    };
    exc.on(0);
    assert_eq!(exc, Exc {
      a  : vec![
        vec![
          Exc1 {
            n: 1,
            v: vec![1.0],
            a: 1.0,
          }],
        vec![
          Exc1 {
            n: 0,
            v: vec![1.0],
            a: 1.0,
          },
          Exc1 {
            n: 1,
            v: vec![0.5],
            a: 1.0,
          }],
      ],
      exi: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
      _p : PhantomData,
    });
    let mut o = vec![Cplxpol { mag: 0.0, angle: 0.0 },
                     Cplxpol { mag: 1.0, angle: 0.0 }];
    exc.tick(&mut o);
    assert_eq!(o,
     vec![Cplxpol { mag: 1.0, angle: 0.0 },
          Cplxpol { mag: 1.5, angle: 0.0 }] );
    assert_eq!(exc, Exc {
      a  : vec![
        vec![
          Exc1 {
            n: 0,
            v: vec![1.0],
            a: 1.0,
          }],
        vec![
          Exc1 {
            n: 0,
            v: vec![1.0],
            a: 1.0,
          },
          Exc1 {
            n: 0,
            v: vec![0.5],
            a: 1.0,
          }],
      ],
      exi: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
      _p : PhantomData,
    });
  }

  #[wasm_bindgen_test(unsupported = test)]
  fn test_exc_harm() {
    use super::Cplxpol;
    use super::Exc1;
    use super::Exc;
    use std::marker::PhantomData;
    let mut exc: Exc<Cplxpol> = Exc {
      a  : vec![
        vec![
          Exc1 {
            n: 0,
            v: vec![1.0],
            a: 1.0,
          }],
        vec![
          Exc1 {
            n: 0,
            v: vec![1.0],
            a: 1.0,
          },
          Exc1 {
            n: 0,
            v: vec![0.5],
            a: 1.0,
          }],
      ],
      exi: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
      _p : PhantomData,
    };
    exc.harm(0, 0.75);
    assert_eq!(exc, Exc {
      a  : vec![
        vec![
          Exc1 {
            n: 0,
            v: vec![1.0],
            a: 0.75,
          }],
        vec![
          Exc1 {
            n: 0,
            v: vec![1.0],
            a: 0.75,
          },
          Exc1 {
            n: 0,
            v: vec![0.5],
            a: 1.0,
          }],
      ],
      exi: vec![vec![(0, 0), (1, 1)],
                vec![(1, 0)]],
      _p : PhantomData,
    });
  }
}
