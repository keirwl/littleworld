/* Implementation of most of:
 * https://www.redblobgames.com/grids/hexagons/implementation.html
 * with flat-topped hexes, odd-q positioning, primarily using axial coords
 */
#![allow(dead_code)]
use derive_more::{Add, Div, From, Mul, Sub};

#[derive(Add, Sub, Mul, Div, From, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HexAxial {
    pub q: i32,
    pub r: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    N = 0,
    NE = 1,
    SE = 2,
    S = 3,
    SW = 4,
    NW = 5,
}

// the actual direction coordinates
const OFFSETS: [HexAxial; 6] = [
    HexAxial { q: 0, r: -1 },
    HexAxial { q: 1, r: -1 },
    HexAxial { q: 1, r: 0 },
    HexAxial { q: 0, r: 1 },
    HexAxial { q: -1, r: 1 },
    HexAxial { q: -1, r: 0 },
];

impl TryFrom<usize> for Direction {
    type Error = usize;
    fn try_from(i: usize) -> Result<Direction, Self::Error> {
        match i {
            0 => Ok(Direction::N),
            1 => Ok(Direction::NE),
            2 => Ok(Direction::SE),
            3 => Ok(Direction::S),
            4 => Ok(Direction::SW),
            5 => Ok(Direction::NW),
            _ => Err(i),
        }
    }
}

impl HexAxial {
    #[must_use]
    pub fn cube(&self) -> (i32, i32, i32) {
        (self.q, self.r, -self.q - self.r)
    }

    #[must_use]
    pub fn from_oddq(col: i32, row: i32) -> HexAxial {
        let parity = col & 1;
        HexAxial {
            q: col,
            r: row - (col - parity) / 2,
        }
    }

    #[must_use]
    pub fn to_oddq(&self) -> (i32, i32) {
        let parity = self.q & 1;
        (self.q, self.r + (self.q - parity) / 2)
    }

    // distance from the origin. Called "length" because
    // "distance" is used for the length between two hexes
    #[must_use]
    pub fn length(&self) -> i32 {
        let (q, r, s) = self.cube();
        (q.abs() + r.abs() + s.abs()) / 2
    }

    // redblobgames showed two equivalent ways of getting length,
    // benchmarking found this one to be slightly slower
    #[must_use]
    pub fn length_max(&self) -> i32 {
        let (q, r, s) = self.cube();
        std::cmp::max(std::cmp::max(q.abs(), r.abs()), s.abs())
    }

    #[must_use]
    pub fn distance(&self, other: HexAxial) -> i32 {
        (*self - other).length()
    }

    #[must_use]
    pub fn neighbour(&self, dir: Direction) -> HexAxial {
        *self + OFFSETS[dir as usize]
    }

    pub fn neighbours(&self) -> impl Iterator<Item = HexAxial> {
        OFFSETS.iter().map(|o| *self + *o)
    }
}

#[derive(Debug)]
pub struct Grid<T> {
    w: i32,
    h: i32,
    store: Vec<T>,
}

#[derive(Debug)]
pub struct GridError(String);
impl std::error::Error for GridError {}
impl std::fmt::Display for GridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// gives us many of the methods on Vec
impl<T> std::ops::Deref for Grid<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.store
    }
}

impl<T> std::ops::DerefMut for Grid<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.store
    }
}

#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
// all casting here is checked as safe in new_empty
impl<T> Grid<T> {
    fn new_empty(w: usize, h: usize) -> Result<Grid<T>, GridError> {
        if !(w > 0 && h > 0) {
            return Err(GridError(
                "Grid width and height cannot be zero".to_string(),
            ));
        }
        let wi = i32::try_from(w)
            .map_err(|_| GridError("Grid width too large (> i32::MAX)".to_string()))?;
        let hi = i32::try_from(h)
            .map_err(|_| GridError("Grid height too large (> i32::MAX)".to_string()))?;
        let len = wi
            .checked_mul(hi)
            .ok_or_else(|| GridError("Grid width * height overflows i32".to_string()))?;
        Ok(Grid {
            w: wi,
            h: hi,
            store: Vec::<T>::with_capacity(len.try_into().unwrap()),
        })
    }

    #[must_use]
    pub fn dimensions(&self) -> (usize, usize) {
        (self.w as usize, self.h as usize)
    }

    fn bound(&self, idx: usize) -> bool {
        idx < self.w as usize * self.h as usize
    }

    // returns None for out-of bounds, rather than wrapping
    fn index(&self, a: HexAxial) -> Option<usize> {
        let (col, row) = a.to_oddq();
        if row < 0 || row >= self.h || col < 0 || col >= self.w {
            return None;
        }
        Some((row * self.w + col) as usize)
    }

    // NOT bounds-checking here: caller must ensure idx < w * h,
    // best done by checking against the grid's bounds.
    // Every produced HexAxial will be checked in index()
    fn at(&self, idx: usize) -> HexAxial {
        let col = idx as i32 % self.w;
        let row = idx as i32 / self.w;
        HexAxial::from_oddq(col, row)
    }

    fn coords(&self, idx: usize) -> Option<(usize, usize)> {
        if !self.bound(idx) {
            return None;
        }
        Some((idx % self.w as usize, idx / self.w as usize))
    }
}

impl<T> Grid<T> {
    pub fn new_filled(w: usize, h: usize, t: T) -> Result<Grid<T>, GridError>
    where
        T: Clone,
    {
        let mut grid: Grid<T> = Grid::new_empty(w, h)?;
        for _ in 0..(w * h) {
            grid.store.push(t.clone());
        }
        Ok(grid)
    }

    pub fn new_with_index(
        w: usize,
        h: usize,
        mut f: impl FnMut(usize) -> T,
    ) -> Result<Grid<T>, GridError> {
        let mut grid: Grid<T> = Grid::new_empty(w, h)?;
        for i in 0..(w * h) {
            grid.store.push(f(i));
        }
        Ok(grid)
    }

    pub fn new_with_coords(
        w: usize,
        h: usize,
        mut f: impl FnMut((usize, usize)) -> T,
    ) -> Result<Grid<T>, GridError> {
        let mut grid: Grid<T> = Grid::new_empty(w, h)?;
        for j in 0..h {
            for i in 0..w {
            grid.store.push(f((i, j)));
            }
        }
        Ok(grid)
    }

    pub fn new_from(
        w: usize,
        h: usize,
        i: impl IntoIterator<Item = T>,
    ) -> Result<Grid<T>, GridError>
    where
        T: Clone,
    {
        let mut grid: Grid<T> = Grid::new_empty(w, h)?;
        grid.store.extend(i.into_iter().take(w * h));
        if grid.store.len() != grid.store.capacity() {
            return Err(GridError(
                "iterator is not big enough to fill grid".to_string(),
            ));
        }
        Ok(grid)
    }

    // returns Err(attempted value) if out-of-bounds
    pub fn set(&mut self, idx: usize, t: T) -> Result<(), T>
    where
        T: Copy,
    {
        if !self.bound(idx) {
            return Err(t);
        }
        self.store[idx] = t;
        Ok(())
    }

    pub fn position(&self, t: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        self.store.iter().position(|x| x == t)
    }

    // can return less than 6 neighbours if idx is at the edge, but cannot take an
    // idx off the grid and return neighbours on it. Instead will return empty
    pub fn neighbours(&self, idx: usize) -> impl Iterator<Item = (Direction, usize)> + use<T> {
        let mut result = Vec::<(Direction, usize)>::new();
        if self.bound(idx) {
            result.extend(
                OFFSETS
                    .into_iter()
                    .enumerate()
                    .map(move |(i, o)| (i.try_into().unwrap(), self.index(self.at(idx) + o)))
                    .filter(|(_, o)| o.is_some())
                    .map(|(i, o)| (i, o.unwrap())),
            );
        }
        result.into_iter()
    }

    // similar to above
    pub fn ring(&self, idx: usize, n: u32) -> impl Iterator<Item = usize> + use<T> {
        let mut result = Vec::<usize>::new();
        if !self.bound(idx) || i32::try_from(n).is_err() {
            return result.into_iter();
        }
        if n == 0 {
            result.push(idx);
        } else {
            let mut hex = self.at(idx) + OFFSETS[4] * i32::try_from(n).unwrap();
            for o in OFFSETS {
                for _ in 0..n {
                    if let Some(i) = self.index(hex) {
                        result.push(i);
                    }
                    hex = hex + o;
                }
            }
        }
        result.into_iter()
    }

    pub fn spiral(&self, idx: usize, n: u32) -> impl Iterator<Item = usize> + use<T>  {
        let mut result = Vec::<usize>::new();
        for i in 0..=n {
            result.extend(self.ring(idx, i));
        }
        result.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn origin() -> HexAxial {
        HexAxial { q: 0, r: 0 }
    }

    #[fixture]
    fn other() -> HexAxial {
        HexAxial { q: 1, r: 2 }
    }

    #[rstest]
    fn test_hex_distance(origin: HexAxial, other: HexAxial) {
        assert_eq!(origin.distance(other), 3);
    }

    #[rstest]
    fn test_hex_distance_reversible(origin: HexAxial, other: HexAxial) {
        assert_eq!(origin.distance(other), other.distance(origin));
    }

    #[rstest]
    fn test_hex_distance_zero(other: HexAxial) {
        assert_eq!(other.distance(other), 0);
    }

    #[rstest]
    #[case(Direction::N, OFFSETS[Direction::N as usize])]
    #[case(Direction::NE, HexAxial { q: 1, r: -1 })]
    #[case(Direction::SE, HexAxial { q: 1, r: 0 })]
    #[case(Direction::S, HexAxial { q: 0, r: 1 })]
    #[case(Direction::SW, HexAxial { q: -1, r: 1 })]
    #[case(Direction::NW, HexAxial { q: -1, r: 0 })]
    fn test_hex_neighbour(origin: HexAxial, #[case] dir: Direction, #[case] hex: HexAxial) {
        assert_eq!(origin.neighbour(dir), hex);
        assert_eq!(origin.distance(hex), 1);
    }

    /* With our flat-topped, odd-q arrangement, a rectangle has single-hex
    * corners at the top left (0, 0) and bottom-right, two-hex corners at
    * the others. Main fixture will be a 16x8 grid with the following values:
    一   三   五   七   九   百   上   左
      二   四   六   八   十   千   下   右
    中   小   日   早   林   川   空   天
      大   月   年   木   山   土   田   生
    花   虫   人   女   子   耳   手   見
      草   犬   名   男   目   口   足   音
    力   円   出   休   夕   文   学   村
      気   入   立   先   本   字   校   町
    森   水   玉   石   糸   車   雨   青
      正   火   王   竹   貝   金   赤   白
    数   少   半   太   広   点   交   角
      多   万   形   細   長   丸   光   計
    直   矢   強   同   母   姉   弟   自
      線   弱   高   親   父   兄   妹   友
    体   頭   首   時   朝   夜   週   夏
      毛   顔   心   曜   昼   分   春   秋
    */

    const GRID_STR: &str = r"
        一二三四五六七八九十百千上下左右
        中大小月日年早木林山川土空田天生
        花草虫犬人名女男子目耳口手足見音
        力気円入出立休先夕本文字学校村町
        森正水火玉王石竹糸貝車金雨赤青白
        数多少万半形太細広長点丸交光角計
        直線矢弱強高同親母父姉兄弟妹自友
        体毛頭顔首心時曜朝昼夜分週春夏秋
    ";

    #[fixture]
    fn kanji_grid() -> Grid<char> {
        Grid::new_from(16, 8, GRID_STR.chars().filter(|c| !c.is_whitespace())).unwrap()
    }

    #[rstest]
    fn test_grid_construction(kanji_grid: Grid<char>) {
        assert_eq!(kanji_grid.dimensions(), (16, 8));
        assert_eq!(kanji_grid.store.len(), 128);
        assert_eq!(kanji_grid.store.capacity(), 128);
    }

    #[rstest]
    fn test_grid_fails_zero() {
        assert!(Grid::new_filled(0, 0, 'a').is_err());
    }

    #[rstest]
    fn test_grid_fails_too_big() {
        assert!(Grid::new_filled(50_000, 50_000, 'a').is_err());
    }

    #[rstest]
    fn test_grid_indexing(kanji_grid: Grid<char>) {
        // basic indexing
        assert_eq!(kanji_grid.at(0), HexAxial { q: 0, r: 0 });
        assert_eq!(kanji_grid.at(18), HexAxial { q: 2, r: 0 });
        assert_eq!(kanji_grid.index(HexAxial { q: 2, r: 0 }), Some(18));
        assert_eq!(kanji_grid.index(HexAxial { q: 15, r: 0 }), Some(127));

        assert_eq!(
            kanji_grid.index(HexAxial { q: 3, r: -1 }.neighbour(Direction::SW)),
            Some(18)
        );
        assert_eq!(
            kanji_grid.index(HexAxial { q: 2, r: -1 }.neighbour(Direction::SW)),
            Some(1)
        );

        // out-of-bounds
        assert_eq!(kanji_grid.index(HexAxial { q: 0, r: 16 }), None);
        assert_eq!(kanji_grid.index(HexAxial { q: 15, r: -8 }), None);
        assert_eq!(kanji_grid.index(HexAxial { q: -1, r: -1 }), None);

        // identity
        for i in 0..=127 {
            assert_eq!(kanji_grid.index(kanji_grid.at(i)), Some(i));
        }
    }

    #[rstest]
    #[case('太', vec!['石', '竹', '細', '同', '形', '王'])]
    #[case('犬', vec!['月', '人', '出', '入', '円', '虫'])]
    fn test_grid_neighbours(
        kanji_grid: Grid<char>,
        #[case] pos: char,
        #[case] expected: Vec<char>,
    ) {
        let neighbours: Vec<char> = kanji_grid
            .neighbours(kanji_grid.position(&pos).unwrap())
            .map(|(_, i)| *kanji_grid.get(i).unwrap())
            .collect();
        assert_eq!(neighbours, expected);
    }

    #[rstest]
    #[case('秋', vec!['友', '夏'])]
    #[case('森', vec!['力', '気', '正', '数'])]
    fn test_grid_neighbours_edge(
        kanji_grid: Grid<char>,
        #[case] pos: char,
        #[case] expected: Vec<char>,
    ) {
        let neighbours: Vec<char> = kanji_grid
            .neighbours(kanji_grid.position(&pos).unwrap())
            .map(|(_, i)| *kanji_grid.get(i).unwrap())
            .collect();
        assert_eq!(neighbours, expected);
    }

    #[rstest]
    #[case(0, vec!['太'])]
    #[case(1, vec!['形', '王', '石', '竹', '細', '同'])]
    #[case(2, vec!['強', '半', '玉', '立', '休', '先', '糸', '広', '母', '親', '時', '高'])]
    fn test_grid_ring(kanji_grid: Grid<char>, #[case] n: u32, #[case] expected: Vec<char>) {
        let ring: Vec<char> = kanji_grid
            .ring(kanji_grid.position(&'太').unwrap(), n)
            .map(|i| *kanji_grid.get(i).unwrap())
            .collect();
        assert_eq!(ring, expected);
    }

    #[rstest]
    #[case(1, vec!['力', '気', '正', '数'])]
    #[case(2, vec!['花', '草', '円', '水', '少', '多', '直'])]
    fn test_grid_ring_edge(kanji_grid: Grid<char>, #[case] n: u32, #[case] expected: Vec<char>) {
        let ring: Vec<char> = kanji_grid
            .ring(kanji_grid.position(&'森').unwrap(), n)
            .map(|i| *kanji_grid.get(i).unwrap())
            .collect();
        assert_eq!(ring, expected);
    }

    #[rstest]
    fn test_grid_spiral(kanji_grid: Grid<char>) {
        let spiral: Vec<char> = kanji_grid
            .spiral(kanji_grid.position(&'太').unwrap(), 2)
            .map(|i| *kanji_grid.get(i).unwrap())
            .collect();
        assert_eq!(
            spiral,
            vec![
                '太', '形', '王', '石', '竹', '細', '同', '強', '半', '玉', '立', '休', '先', '糸',
                '広', '母', '親', '時', '高'
            ]
        );
    }

    #[rstest]
    fn test_grid_spiral_edge(kanji_grid: Grid<char>) {
        let spiral: Vec<char> = kanji_grid
            .spiral(kanji_grid.position(&'森').unwrap(), 2)
            .map(|i| *kanji_grid.get(i).unwrap())
            .collect();
        assert_eq!(
            spiral,
            vec![
                '森', '力', '気', '正', '数', '花', '草', '円', '水', '少', '多', '直'
            ]
        );
    }
}
