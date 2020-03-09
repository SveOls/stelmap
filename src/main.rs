use bmp;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use std::{collections::BTreeMap, fmt::Debug, time::Duration};

mod saveread;

#[derive(Debug)]
struct Img {
    image: Vec<u32>,
    width: usize,
    height: usize,
    objects: BTreeMap<usize, Vec<((usize, usize), FromImage)>>,
    selected: Option<((usize, usize), FromImage)>,
}

impl Img {
    fn new(width: usize, height: usize) -> Img {
        let mut ret = Img {
            image: Vec::new(),
            width,
            height,
            objects: BTreeMap::new(),
            selected: None,
        };
        for _ in 0..ret.dim() {
            ret.push(0xFFFFFF)
        }
        ret.set_background(0x666666);
        ret.set_menu(0xF0F0F0);
        ret
    }
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    fn dim(&self) -> usize {
        self.height * self.width
    }
    fn push(&mut self, val: u32) {
        self.image.push(val);
    }
    fn get_img(&self) -> &[u32] {
        &self.image
    }
    fn set_background(&mut self, inp: u32) {
        self.objects.insert(
            0,
            vec![(
                (0, 0),
                FromImage {
                    content: vec![vec![inp; self.width]; self.height],
                    selectable: false,
                    name: None,
                },
            )],
        );
    }
    fn set_menu(&mut self, inp: u32) {
        self.objects.insert(
            1,
            vec![(
                (0, 0),
                FromImage::from_vec(
                    vec![inp; (self.width - self.height) * self.height],
                    self.width - self.height,
                    false,
                ),
            )],
        );
    }

    fn attach(&mut self, object: &FromImage, coord: (usize, usize), layer: Option<usize>) {
        let temp = if let Some(a) = layer {
            self.objects.entry(a).or_insert(Vec::new())
        } else {
            if let Some(&a) = self.objects.keys().max() {
                self.objects.entry(a + 1).or_insert(Vec::new())
            } else {
                self.objects.entry(1).or_insert(Vec::new())
            }
        };
        (*temp).push((coord, object.to_format()));
    }
    fn update(&mut self) {
        for (_, objects) in self.objects.iter() {
            for ((x, y), object) in objects {
                // println!("{} {}", object.len(), object[0].len());
                for i in 0..object.len() {
                    for j in 0..object[0].len() {
                        if object[i][j] / 0x1000000 < 1 && object[i][j] != 0xFEFEFE {
                            self.image[(*y + i) * self.width + (*x + j)] = object[i][j];
                        }
                    }
                }
            }
        }
        if let Some(((x, y), object)) = &self.selected {
            for i in 0..object.len() {
                for j in 0..object[0].len() {
                    if j == 49 {
                        // println!("{:.x}", object[i][j]);
                    }
                    if object[i][j] / 0x1000000 < 1 && object[i][j] != 0xFEFEFE {
                        self.image[(*y + i) * self.width + (*x + j)] = object[i][j];
                        // println!("{} {} {}", object[i][j], i, j);
                    }
                }
            }
        }
    }
    fn select(&mut self, input: ((usize, usize), (usize, usize))) {
        let (pos, dim) = input;
        let mut ret = Vec::new();
        ret.push(vec![0x0; dim.0]);
        let mut temp = vec![0x0];
        temp.append(&mut vec![0x1000000; dim.0 - 2]);
        temp.push(0x0);
        for _ in 2..dim.1 {
            ret.push(temp.clone());
        }
        ret.push(vec![0x0; dim.0]);
        self.selected = Some((
            pos,
            FromImage {
                content: ret,
                selectable: false,
                name: None,
            },
        ));
    }

    fn get_item(&self, pos: (usize, usize)) -> Option<(usize, usize)> {
        let mut i = 0;
        for (j, vals) in self.objects.iter().rev() {
            for ((x, y), object) in vals {
                if pos.0 < object[0].len() + x
                    && pos.0 > *x
                    && pos.1 < object.len() + y
                    && pos.1 > *y
                    && object.selectable
                {
                    if let Some(a) = object.get_name() {
                        println!("{}", a);
                    }
                    return Some((*j, i));
                }
                i += 1;
            }
            i = 0;
        }
        None
    }
    ///returns (position, width/height)
    fn obdim(&self, id: usize, pos: usize) -> ((usize, usize), (usize, usize)) {
        let (pos, a) = &self.objects[&id][pos];
        // println!("{}, {}, {}, {}", a[0].len(), pos.0, a.len(), pos.1);
        let ext = (a[0].len(), a.len());
        (*pos, ext)
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
struct FromImage {
    content: Vec<Vec<u32>>,
    selectable: bool,
    name: Option<String>,
}

impl FromImage {
    fn new(inp: &str, selectable: bool) -> Result<FromImage, Box<dyn std::error::Error>> {
        let mut content = Vec::new();

        let inp = bmp::open(inp).expect("OK");
        let width = inp.get_width() - 1;

        let mut pix: bmp::Pixel;
        let mut temp = Vec::new();
        for (x, y) in inp.coordinates() {
            pix = inp.get_pixel(x, y);
            temp.push((pix.r as u32 * 256 * 256) + (pix.g as u32 * 256) + (pix.b as u32));
            if x == width {
                content.push(temp.clone());
                temp.clear();
            }
        }
        Ok(FromImage {
            content,
            selectable,
            name: None,
        })
    }
    fn from_vec(content2: Vec<u32>, width: usize, selectable: bool) -> FromImage {
        let mut content = Vec::new();
        for i in 0..content2.len() / width {
            content.push((0..width).map(|x| content2[x + i * width]).collect())
        }
        FromImage {
            content,
            selectable,
            name: None,
        }
    }
    fn len(&self) -> usize {
        self.content.len()
    }
    fn to_format(&self) -> FromImage {
        self.clone()
    }
    fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }
    fn get_name(&self) -> &Option<String> {
        &self.name
    }
}

impl std::ops::Index<usize> for FromImage {
    type Output = Vec<u32>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.content[index]
    }
}

// #[derive(Debug, Clone)]
// pub struct GalObject {
//     id: usize,
//     x: i64,
//     y: i64,
//     typ: String,
//     name: String,
//     planets: Vec<usize>
// }

fn make_shit(mut w: usize, mut h: usize) -> Result<(), Box<dyn std::error::Error>> {
    let smiley_tester = &FromImage::from_vec(
        vec![
            0x1000000, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0x1000000, 0xFFFFFF, 0x0,
            0x0, 0xFFFFFF, 0x0, 0x0, 0xFFFFFF, 0xFFFFFF, 0x0, 0x0, 0xFFFFFF, 0x0, 0x0, 0xFFFFFF,
            0xFFFFFF, 0x0, 0x0, 0xFFFFFF, 0x0, 0x0, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF,
            0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF,
            0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0x0, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0x0,
            0xFFFFFF, 0xFFFFFF, 0x0, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0x0, 0xFFFFFF, 0xFFFFFF, 0x0,
            0x0, 0x0, 0x0, 0x0, 0xFFFFFF, 0x1000000, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF,
            0xFFFFFF, 0x1000000,
        ],
        7,
        true,
    );

    let a = saveread::reader()?;
    println!("finished reading");

    let (_, galaxy) = a.get_obj_iter().next().unwrap();
    let mut objit = galaxy.get_obj_iter(); // iterator over GalObjects ( in vector )
    let mut maxx = 0f64;
    let mut minx = 0f64;
    let mut maxy = 0f64;
    let mut miny = 0f64;
    while let Some(a) = objit.next() {
        if a.gx() > maxx {
            maxx = a.gx();
        } else if a.gx() < minx {
            minx = a.gx();
        }
        if a.gy() > maxy {
            maxy = a.gy();
        } else if a.gy() < miny {
            miny = a.gy();
        }
    }
    let scale = 0.9 * (h as f64) / (maxx - minx).max(maxy - miny);
    let mx = -minx - 0.05 * (h as f64) + 300f64;
    let my = -miny - 0.05 * (h as f64);
    println!("{} {} {} {}", minx, maxx, miny, maxy);
    println!("{} {} {}", scale, mx, my);

    if h > w {
        std::mem::swap(&mut w, &mut h);
    }
    let star = &mut FromImage::new("./data/star.bmp", true)?;
    let ua = &mut FromImage::new("./letters/ua.bmp", false)?;
    let la = &mut FromImage::new("./letters/la.bmp", false)?;
    let ln = &mut FromImage::new("./letters/ln.bmp", false)?;

    let mut img = Img::new(w, h);

    let mut window = Window::new(
        "test",
        img.width(),
        img.height(),
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )?;
    img.attach(ua, (10, 10), Some(3));
    img.attach(la, (18, 10), Some(3));
    img.attach(la, (26, 10), Some(3));
    img.attach(ln, (34, 10), Some(3));


    let mut objit = galaxy.get_obj_iter();
    while let Some(a) = objit.next() {
        let x = ((a.gx() * scale) + mx) as usize;
        let y = ((a.gy() * scale) + my) as usize;
        star.set_name(Some(String::from(a.get_name())));
        img.attach(star, (x, y), Some(3));
    }
    star.name = None;

    let mut TEMP = 0;

    let mut keep = true;
    let mut change = true;
    while window.is_open() && keep {
        // img.resize(window.get_size());
        window.limit_update_rate(Some(Duration::from_millis(20)));
        if change {
            img.update();
            window.update_with_buffer(img.get_img(), img.width(), img.height())?;
        // gotta switch so it only updates when needed
        } else {
            window.update();
        }
        change = false;

        if let Some(coordinates) = window.get_mouse_pos(MouseMode::Clamp) {
            let (x, y) = (coordinates.0 as usize, coordinates.1 as usize);

            if window.get_mouse_down(MouseButton::Left) {
                // img.attach(smiley_tester, (x, y), Some(2));
                // img.select((x, y), (50, 100));
                if let Some((id, pos)) = img.get_item((x, y)) {
                    // let a = img.obdim(id, pos);
                    img.select(img.obdim(id, pos));
                }
                change = true;
            } else if window.get_mouse_down(MouseButton::Right) {
                if TEMP % 2 == 0 {
                    img.attach(star, (x, y), Some(3));
                    img.set_background(0xFF);
                } else {
                    img.attach(smiley_tester, (x, y), Some(3));
                    img.set_background(0xFF00);
                }
                TEMP += 1;
                change = true;
            }
        }

        for keys in window.get_keys_pressed(KeyRepeat::No) {
            for t in keys {
                match t {
                    Key::Escape => keep = false,
                    _ => (),
                }
            }
        }

        window.get_mouse_pos(MouseMode::Clamp).map(|_mouse| {
            // println!("x {} y {}", mouse.0, mouse.1);
        });
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    make_shit(900, 600)?;
    Ok(())
}
