use glob::glob;
use std::{
    collections::HashMap,
    error::Error,
    fmt,
    fs::File,
    io::prelude::*,
    io::{self, BufReader},
    ops,
    path::PathBuf,
};
use zip;

pub fn reader() -> Result<Everything, Box<dyn Error>> {
    let mut world = read()?;
    let a = get_gamestates()?;
    for i in a.iter() {
        if let Some(b) = save_analyser(i, &mut world)? {
            world.push(b);
        }
    }
    // world.save(&mut File::create("save.txt")?)?;
    // let world = read()?;
    world.save(&mut File::create("save.txt")?)?;

    let mut max_x = 0;
    let mut min_x = 0;
    let mut max_y = 0;
    let mut min_y = 0;
    for i in world[0].obj.iter() {
        if i.x > max_x {
            max_x = i.x;
        }
        if i.x < min_x {
            min_x = i.x;
        }
        if i.y > max_y {
            max_y = i.y;
        }
        if i.y < min_y {
            min_y = i.y;
        }
    }
    for i in world.times.values_mut() {
        i.update_max([min_x, max_x, min_y, max_y]);
    }
    // println!("{}, {}", min_x, max_x);
    // println!("{}, {}", min_y, max_x);
    Ok(world)
}

///enum of Ethic types for easier manipulation (than strings)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ethic {
    Gestalt,
    Spiritualist,
    Materialist,
    Egalitarian,
    Authoritarian,
    Pacifist,
    Militarist,
    Xenophile,
    Xenophobe,
    Non(String),
}

impl Ethic {
    fn read_to_ethic(inp: &str) -> Ethic {
        match inp {
            "Gestalt" => Ethic::Gestalt,
            "Spiritualist" => Ethic::Spiritualist,
            "Materialist" => Ethic::Materialist,
            "Egalitarian" => Ethic::Egalitarian,
            "Authoritarian" => Ethic::Authoritarian,
            "Pacifist" => Ethic::Pacifist,
            "Militarist" => Ethic::Militarist,
            "Xenophile" => Ethic::Xenophile,
            "Xenophobe" => Ethic::Xenophobe,
            _ => Ethic::Non(inp.to_owned()),
        }
    }
    fn str_to_ethic(inp: &str) -> Ethic {
        match inp {
            "ethic_gestalt_consciousness" => Ethic::Gestalt,
            "ethic_spiritualist" => Ethic::Spiritualist,
            "ethic_materialist" => Ethic::Materialist,
            "ethic_egalitarian" => Ethic::Egalitarian,
            "ethic_authoritarian" => Ethic::Authoritarian,
            "ethic_pacifist" => Ethic::Pacifist,
            "ethic_militarist" => Ethic::Militarist,
            "ethic_xenophile" => Ethic::Xenophile,
            "ethic_xenophobe" => Ethic::Xenophobe,
            _ => Ethic::Non(inp.to_owned()),
        }
    }
}

impl fmt::Display for Ethic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ethic::Gestalt => write!(f, "Gestalt"),
            Ethic::Spiritualist => write!(f, "Spiritualist"),
            Ethic::Materialist => write!(f, "Materialist"),
            Ethic::Egalitarian => write!(f, "Egalitarian"),
            Ethic::Authoritarian => write!(f, "Authoritarian"),
            Ethic::Pacifist => write!(f, "Pacifist"),
            Ethic::Militarist => write!(f, "Militarist"),
            Ethic::Xenophile => write!(f, "Xenophile"),
            Ethic::Xenophobe => write!(f, "Xenophobe"),
            Ethic::Non(value) => write!(f, "other: {}", value),
        }
    }
}

///currently only contains a vector of Galaxy, so member functions can be implemented.
#[derive(Debug, Clone)]
pub struct Everything {
    times: HashMap<usize, Galaxy>,
}

impl Everything {
    ///returns Everything with its only member value initialized to an empty vector of Galaxy.
    fn new() -> Everything {
        Everything {
            times: HashMap::new(),
        }
    }
    ///pushes a Galaxy to the Everything-object.
    fn push(&mut self, inp: Galaxy) {
        self.times.insert(inp.date, inp);
    }
    fn read(it: &mut impl Iterator<Item = String>) -> Result<Everything, Box<dyn Error>> {
        let mut ret = Everything::new();
        while let Some(inp) = Galaxy::read(it)? {
            ret.times.insert(inp.date, inp);
        }
        Ok(ret)
    }
    fn save(&self, save: &mut File) -> Result<(), Box<dyn Error>> {
        for i in self.times.values() {
            i.save(save)?;
        }
        Ok(())
    }
    pub fn get_obj_iter(&self) -> impl Iterator<Item = (&usize, &Galaxy)> {
        self.times.iter()
    }
}

impl ops::Index<usize> for Everything {
    type Output = Galaxy;
    fn index(&self, ind: usize) -> &Self::Output {
        &self.times[&ind]
    }
}

///the Galaxy object contains all info collected from a single save,
/// in the form of a Date (usize), a vector of Empire, and a vector of Species.
#[derive(Debug, Clone)]
pub struct Galaxy {
    maxc: [f64; 4],
    date: usize,
    empires: Vec<Empire>,
    species: Vec<Species>,
    obj: Vec<GalObject>,
}

impl Galaxy {
    pub fn maxc(&self) -> [f64; 4] {
        self.maxc
    }
    fn update_max(&mut self, inp: [i64; 4]) {
        self.maxc = [
            inp[0] as f64 / 1000f64,
            inp[1] as f64 / 1000f64,
            inp[2] as f64 / 1000f64,
            inp[3] as f64 / 1000f64,
        ];
    }
    pub fn get_obj_iter(&self) -> impl Iterator<Item = &GalObject> {
        self.obj.iter()
    }
    ///returns an empty Galaxy-object; date = 0, and empires+species+obj are empty vectors.
    fn new() -> Galaxy {
        Galaxy {
            maxc: [0.0; 4],
            date: 0,
            empires: Vec::new(),
            species: Vec::new(),
            obj: Vec::new(),
        }
    }
    fn read(it: &mut impl Iterator<Item = String>) -> Result<Option<Galaxy>, Box<dyn Error>> {
        let mut ret = Galaxy::new();
        match it.next() {
            Some(a) => ret
                .setdate2(a.get(..10).expect("Error in setdate, Galaxy1"))
                .expect("Error in setdate, Galaxy2"),
            None => return Ok(None),
        }
        while let Some(line) = it.next() {
            match line.get(..) {
                Some("\tspecies {") => {
                    while let Some(spec) = Species::read(it)? {
                        ret.species.push(spec);
                    }
                }
                Some("\tempires {") => {
                    while let Some(emp) = Empire::read(it)? {
                        ret.empires.push(emp);
                    }
                }
                Some("\tobjects {") => {
                    while let Some(obj) = GalObject::read(it)? {
                        ret.obj.push(obj);
                    }
                }
                Some("}") => break,
                Some(a) => panic!(
                    "Error in reading galaxy: this shouldn't happem >{}< and >{}<",
                    a,
                    it.next().unwrap()
                ),
                None => panic!("NOOO"),
            }
        }
        Ok(Some(ret))
    }
    ///pushes an Empire into the Galaxy object.
    fn push(&mut self, inp: Empire) {
        self.empires.push(inp);
    }
    ///adds all galactic objects to the Galaxy
    fn push_g(&mut self, inp: Vec<GalObject>) {
        self.obj = inp;
    }
    ///turns the save date into an usize; aka the number of days since day 0 (2200.01.01)
    fn setdate(&mut self, inp: &str) -> Result<(), Box<dyn Error>> {
        self.date += 12 * 30 * (parser(&inp[6..10])? - 2200);
        self.date += 30 * (parser(&inp[11..13])? - 1);
        self.date += parser(&inp[14..16])? - 1;
        Ok(())
    }
    fn setdate2(&mut self, inp: &str) -> Result<(), Box<dyn Error>> {
        self.date += 12 * 30 * (parser(&inp[..4])? - 2200);
        self.date += 30 * (parser(&inp[5..7])? - 1);
        self.date += parser(&inp[8..10])? - 1;
        Ok(())
    }
    ///takes a mutable reference to an iterator over the file,
    /// and reads the area between "species={ ... }", saving the contents in
    /// the species-field of a Galaxy object.
    fn get_species<'a>(&mut self, inp: &mut impl Iterator<Item = &'a str>) {
        while let Some(line1) = inp.next() {
            let mut next = Species::new();
            if line1 == "}" {
                break;
            } else if line1 == "\t{" {
                while let Some(line2) = inp.next() {
                    match line2.get(2..7) {
                        Some("name=") => next.set_name(line2),
                        Some("plura") => next.set_plural(line2),
                        Some("adjec") => next.set_adjective(line2),
                        Some("portr") => next.set_portrait(line2),
                        Some("trait") => next.set_traits(inp),
                        _ => {
                            if line2 == "\t}" {
                                break;
                            }
                        }
                    }
                }
            } else {
                panic!(
                    "Whats happening here: 1: {}, 2: {}, 3: {}",
                    line1,
                    inp.next().unwrap(),
                    inp.next().unwrap()
                );
            }
            self.species.push(next);
        }
    }
    fn date_legible(&self) -> String {
        let year = self.date / 360;
        let month = (self.date / 30) - (12 * year);
        let day = self.date - (month * 30) - (year * 360);
        match day % 10 {
            0 => match month % 10 {
                0 => format!("{}.0{}.0{}", year + 2200, month + 1, day + 1),
                _ => format!("{}.{}.0{}", year + 2200, month + 1, day + 1),
            },
            _ => match month % 10 {
                0 => format!("{}.0{}.{}", year + 2200, month + 1, day + 1),
                _ => format!("{}.{}.{}", year + 2200, month + 1, day + 1),
            },
        }
    }
    fn save(&self, save: &mut File) -> Result<(), Box<dyn Error>> {
        save.write_all(format!("{} {{\n\tspecies {{\n", self.date_legible()).as_bytes())?;
        for (num, i) in self.species.iter().enumerate() {
            i.save(num, save)?;
        }
        save.write_all(b"\t}\n\tempires {\n")?;
        for i in self.empires.iter() {
            i.save(save)?;
        }
        save.write_all(b"\t}\n\tobjects {\n")?;
        for i in self.obj.iter() {
            i.save(save)?;
        }
        save.write_all(b"\t}\n}\n")?;
        Ok(())
    }
}

///An empire, containing all planets it controls. Also ethics if applicable.
#[derive(Clone)]
struct Empire {
    id: usize,
    name: String,
    adjective: String,
    planets: Vec<Planet>,
    ethics: Option<[Ethic; 3]>,
    color: [Option<String>; 4],
}

impl Empire {
    fn newe() -> Empire {
        Empire {
            id: std::usize::MAX,
            name: String::new(),
            adjective: String::new(),
            planets: Vec::new(),
            ethics: None,
            color: [None, None, None, None],
        }
    }
    ///Returns an Empire object with no planets and no ethics.
    fn new<'a>(
        it: &mut impl Iterator<Item = &'a str>,
        plan: &mut HashMap<usize, Planet>,
    ) -> Result<Option<Empire>, String> {
        let id = match it.next() {
            Some("}") => return Ok(None),
            Some(line) => match line.get(1..(line.chars().count() - 2)) {
                Some(a) => parser(a)?,
                None => return Err(format!("Couldn't parse Emprie ID: >{}<", line)),
            },
            None => return Err(format!("Couldn't parse Empire ID; end of iterator")),
        };
        let mut ret = Empire {
            id: id,
            name: String::new(),
            adjective: String::new(),
            planets: Vec::new(),
            ethics: None,
            color: [None, None, None, None],
        };
        while let Some(line) = it.next() {
            if line == "\t}" {
                break;
            }
            match line.get(2..8) {
                Some("\tcolor") => {
                    for i in 0..4 {
                        let line = match it.next() {
                            Some(a) => a,
                            None => {
                                return Err(format!(
                                    "Cannot iterate when collecting colors for empires"
                                ))
                            }
                        };
                        ret.color[i] = match line.get(5..(line.chars().count() - 1)) {
                            Some(a) => str_to_color(a),
                            None => return Err(format!("Cannot index into color for empires")),
                        }
                    }
                }
                Some("name=\"") => {
                    ret.name = match line.get(8..(line.chars().count() - 1)) {
                        Some(a) => a.to_owned(),
                        None => {
                            return Err(format!("Cannot get name from line >{}< for empires", line))
                        }
                    }
                }
                Some("adject") => {
                    ret.adjective = match line.get(13..(line.chars().count() - 1)) {
                        Some(a) => a.to_owned(),
                        None => {
                            return Err(format!(
                                "Cannot get acjective from line >{}< for empires",
                                line
                            ))
                        }
                    }
                }
                Some("ethos=") => {
                    let mut i = 0;
                    let mut temp = [Ethic::Gestalt, Ethic::Gestalt, Ethic::Gestalt];
                    while let Some(line2) = it.next() {
                        if line2 == "\t\t}" {
                            temp[2] = temp[1].clone();
                            break;
                        }
                        match line2.get(10..(line2.chars().count() - 1)) {
                            Some(a) => match line2.get(16..23) {
                                Some("fanatic") => {
                                    match line2.get(24..(line2.chars().count() - 1)) {
                                        Some(b) => {
                                            temp[i] = Ethic::str_to_ethic(&format!("ethic_{}", b));
                                            i += 1;
                                            temp[i] = Ethic::str_to_ethic(&format!("ethic_{}", b));
                                            i += 1;
                                        }
                                        None => {
                                            return Err(format!(
                                                "Error for fanatic ethic for: >{}<",
                                                line2
                                            ))
                                        }
                                    }
                                }
                                _ => {
                                    temp[i] = Ethic::str_to_ethic(a);
                                    if temp[i] == Ethic::Gestalt {
                                        break;
                                    }
                                    i += 1;
                                }
                            },
                            None => {
                                return Err(format!("Cannot get ethic at empire from: >{}<", line2))
                            }
                        }
                        if i == 3 {
                            break;
                        }
                    }
                    ret.ethics = Some(temp);
                }
                Some("owned_") => match it.next() {
                    Some(line2) => {
                        match line2.get(3..(line2.chars().count() - 1)) {
                            Some(a) => {
                                for i in a.split(' ') {
                                    ret.planets.push(match plan.remove(&parser(i)?) {
                                        Some(a) => a,
                                        None => continue,
                                    })
                                }
                            }
                            None => return Err(format!("Found no owner planets: >{}<", line2)),
                        };
                    }
                    None => {
                        return Err(format!("Couldn't get owned planets"));
                    }
                },
                _ => {}
            }
        }
        Ok(Some(ret))
    }
    fn read(it: &mut impl Iterator<Item = String>) -> Result<Option<Empire>, Box<dyn Error>> {
        let mut ret = Empire::newe();
        match it.next().unwrap().get(..).unwrap() {
            "\t}" => return Ok(None),
            a => {
                ret.id = parser(
                    a.get(2..(a.chars().count() - 2))
                        .expect("Error in setdate, ??, Galaxy1"),
                )?
            }
        }
        if let Some(a) = it.next().unwrap().get(..) {
            ret.name = a
                .get(3..a.chars().count())
                .expect("Error in setdate, Galaxy1")
                .to_owned();
        }
        if let Some(a) = it.next().unwrap().get(..) {
            ret.adjective = a
                .get(3..a.chars().count())
                .expect("Error in setdate, Galaxy1")
                .to_owned();
        }
        it.next();
        ret.ethics = match it.next().unwrap().get(..) {
            Some("\t\t\t\tNone") => None,
            Some("\t\t\t\tGestalt") => Some([Ethic::Gestalt, Ethic::Gestalt, Ethic::Gestalt]),
            Some(a) => Some([
                Ethic::read_to_ethic(a.get(4..).unwrap()),
                Ethic::read_to_ethic(&it.next().unwrap().get(4..).unwrap()),
                Ethic::read_to_ethic(&it.next().unwrap().get(4..).unwrap()),
            ]),
            None => panic!("Despacito"),
        };
        it.next();
        it.next();
        while let Some(emp) = Planet::read(it)? {
            ret.planets.push(emp);
        }
        it.next();
        for i in 0..4 {
            match it.next().unwrap().get(4..) {
                Some("None") => {}
                Some(a) => ret.color[i] = Some(a.to_owned()),
                _ => panic!("Des?pa?cito"),
            }
        }
        it.next();
        it.next();
        Ok(Some(ret))
    }
    fn save(&self, save: &mut File) -> Result<(), Box<dyn Error>> {
        save.write_all(
            format!(
                "\t\t{} {{\n\t\t\t{}\n\t\t\t{}\n\t\t\tethics {{\n",
                self.id, self.name, self.adjective
            )
            .as_bytes(),
        )?;
        match &self.ethics {
            Some(a) => {
                if a[0] == Ethic::Gestalt {
                    save.write_all(b"\t\t\t\tGestalt\n")?;
                } else {
                    for i in a.iter() {
                        save.write_all(format!("\t\t\t\t{}\n", i).as_bytes())?;
                    }
                }
            }
            None => save.write_all(b"\t\t\t\tNone\n")?,
        }
        save.write_all(b"\t\t\t}\n\t\t\tplanets {\n")?;
        for i in self.planets.iter() {
            i.save(save)?;
        }
        save.write_all(b"\t\t\t}\n\t\t\tcolors {\n")?;
        for i in self.color.iter() {
            match i {
                Some(a) => save.write_all(format!("\t\t\t\t{}\n", a).as_bytes())?,
                None => save.write_all(b"\t\t\t\tNone\n")?,
            }
        }
        save.write_all(b"\t\t\t}\n\t\t}\n")?;
        Ok(())
    }
}

///a planet; id in usize, name in string, type in string, size in usize, population in vec of Pop
#[derive(Clone)]
struct Planet {
    id: usize,
    name: String,
    typ: String,
    size: usize,
    population: Vec<Pop>,
}

impl Planet {
    ///returns an empty planet
    fn new<'a>(
        it: &mut impl Iterator<Item = &'a str>,
        pops: &mut HashMap<usize, Vec<Pop>>,
    ) -> Result<Option<(Planet, bool)>, String> {
        let id = match it.next() {
            Some("\t}") => return Ok(None),
            Some(line) => match line.get(2..(line.chars().count() - 2)) {
                Some(a) => parser(a)?,
                None => return Err(format!("Couldn't parse planet ID: >{}<", line)),
            },
            None => return Err(format!("Couldn't parse planet ID; end of iterator")),
        };
        let mut ret = Planet {
            id: id,
            population: match pops.remove(&id) {
                Some(a) => a,
                None => Vec::new(),
            },
            name: String::new(),
            typ: String::new(),
            size: 0,
        };
        while let Some(line) = it.next() {
            if line == "\t\t}" {
                break;
            }
            match line.get(3..8) {
                Some("name=") => {
                    ret.name = match line.get(9..(line.chars().count() - 1)) {
                        Some(a) => a.to_owned(),
                        None => return Err(format!("Couldn't get the name from line: >{}<", line)),
                    }
                }
                Some("plane") => match line.get(10..14) {
                    Some("size") => {
                        ret.size = match line.get(15..line.chars().count()) {
                            Some(a) => parser(a)?,
                            None => {
                                return Err(format!(
                                    "Couldn't get the planet size from line: >{}<",
                                    line
                                ))
                            }
                        }
                    }
                    Some("clas") => {
                        ret.typ = match line.get(17..(line.chars().count() - 1)) {
                            Some(a) => a.to_owned(),
                            None => {
                                return Err(format!(
                                    "Couldn't get the planet class from line: >{}<",
                                    line
                                ))
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(match ret.population.len() {
            0 => Some((ret, false)),
            _ => Some((ret, true)),
        })
    }
    fn newe() -> Planet {
        Planet {
            id: std::usize::MAX,
            population: Vec::new(),
            name: String::new(),
            typ: String::new(),
            size: 0,
        }
    }
    fn read(it: &mut impl Iterator<Item = String>) -> io::Result<Option<Planet>> {
        let mut ret = Planet::newe();
        ret.id = match it.next().unwrap().get(..) {
            Some("\t\t\t}") => return Ok(None),
            Some(a) => parser(
                a.get(4..(a.chars().count() - 2))
                    .expect("Error in planet id , planet"),
            )
            .expect("Test"),
            None => panic!("fuck"),
        };
        ret.name = match it.next().unwrap().get(..) {
            Some(a) => a
                .get(5..a.chars().count())
                .expect("Error in planet id , planet")
                .to_owned(),
            None => panic!("fuck"),
        };
        ret.typ = match it.next().unwrap().get(..) {
            Some(a) => a
                .get(5..a.chars().count())
                .expect("Error in planet id , planet")
                .to_owned(),
            None => panic!("fuck"),
        };
        ret.size = match it.next().unwrap().get(..) {
            Some(a) => parser(
                a.get(5..a.chars().count())
                    .expect("Error in planet id , planet"),
            )
            .unwrap(),
            None => panic!("fuck"),
        };
        it.next();
        while let Some(a) = Pop::read(it)? {
            ret.population.push(a);
        }
        it.next();
        Ok(Some(ret))
    }
    fn save(&self, save: &mut File) -> io::Result<()> {
        save.write_all(
            format!(
                "\t\t\t\t{} {{\n\t\t\t\t\t{}\n\t\t\t\t\t{}\n\t\t\t\t\t{}\n\t\t\t\t\tpops {{\n",
                self.id, self.name, self.typ, self.size
            )
            .as_bytes(),
        )?;
        for i in self.population.iter() {
            i.save(save)?;
        }
        save.write_all(b"\t\t\t\t\t}\n\t\t\t\t}\n")?;
        Ok(())
    }
}

///a pop.
#[derive(Clone)]
struct Pop {
    id: usize,
    species: usize,
    ethic: Ethic,
    job: String,
    category: String,
    slave: bool,
}

impl Pop {
    fn new<'a>(it: &mut impl Iterator<Item = &'a str>, id: usize) -> Result<(usize, Pop), String> {
        let mut ret = Pop {
            id: id,
            species: std::usize::MAX,
            ethic: Ethic::Gestalt,
            job: String::new(),
            category: String::new(),
            slave: false,
        };
        let mut planet = std::usize::MAX;
        while let Some(line) = it.next() {
            match line.get(..7) {
                Some("\t\tspeci") => {
                    ret.species = match line.get(16..(line.chars().count())) {
                        Some(a) => parser(a)?,
                        None => return Err(format!("Couldn't get species id from: >{}<", line)),
                    }
                }
                Some("\t\t\tethi") => {
                    ret.ethic = match line.get(10..(line.chars().count() - 1)) {
                        Some(a) => Ethic::str_to_ethic(a),
                        None => return Err(format!("Couldn't get ethics from: >{}<", line)),
                    }
                }
                Some("\t\tjob=\"") => {
                    ret.job = match line.get(7..(line.chars().count() - 1)) {
                        Some(a) => a.to_owned(),
                        None => return Err(format!("Couldn't get job from: >{}<", line)),
                    }
                }
                Some("\t\tcateg") => {
                    ret.category = match line.get(12..(line.chars().count() - 1)) {
                        Some(a) => a.to_owned(),
                        None => return Err(format!("Couldn't get category from: >{}<", line)),
                    }
                }
                Some("\t\tplane") => {
                    planet = match line.get(9..(line.chars().count())) {
                        Some(a) => parser(a)?,
                        None => return Err(format!("Couldn't get planet id from: >{}<", line)),
                    }
                }
                Some("\t\tensla") => {
                    ret.slave = match line.get(11..(line.chars().count())) {
                        Some(a) => str_to_bool(a)?,
                        None => return Err(format!("Couldn't get enslavement from: >{}<", line)),
                    }
                }
                None => {
                    if line == "\t}" {
                        break;
                    }
                }
                _ => {}
            }
        }
        Ok((planet, ret))
    }
    fn newe() -> Pop {
        Pop {
            id: std::usize::MAX,
            species: std::usize::MAX,
            ethic: Ethic::Gestalt,
            job: String::new(),
            category: String::new(),
            slave: false,
        }
    }
    fn read(it: &mut impl Iterator<Item = String>) -> io::Result<Option<Pop>> {
        match it.next().unwrap().get(..) {
            Some("\t\t\t\t\t}") => Ok(None),
            Some(a) => {
                let mut ret = Pop::newe();
                let mut temp = a.split("\t");
                ret.id = parser(temp.next().unwrap()).unwrap();
                ret.species = parser(temp.next().unwrap()).unwrap();
                ret.ethic = Ethic::read_to_ethic(temp.next().unwrap());
                ret.job = temp.next().unwrap().to_owned();
                ret.category = temp.next().unwrap().to_owned();
                ret.slave = match temp.next().unwrap().get(..) {
                    Some("false") => false,
                    Some("true") => true,
                    _ => panic!("Despacito, but bool"),
                };
                Ok(Some(ret))
            }
            None => panic!("D?+++spacito"),
        }
    }
    fn save(&self, save: &mut File) -> io::Result<()> {
        save.write_all(
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t\n",
                self.id, self.species, self.ethic, self.job, self.category, self.slave
            )
            .as_bytes(),
        )?;
        Ok(())
    }
}

///species has fields for: name, plural, adjective, portrait, and a vector of traits (string).
#[derive(Debug, Clone)]
struct Species {
    name: String,
    plural: String,
    adjective: String,
    portrait: String,
    traits: Vec<String>,
}

impl Species {
    ///returns an empty species
    fn new() -> Species {
        Species {
            name: String::new(),
            plural: String::new(),
            adjective: String::new(),
            portrait: String::new(),
            traits: Vec::new(),
        }
    }
    fn read(it: &mut impl Iterator<Item = String>) -> io::Result<Option<Species>> {
        let mut ret = Species::new();
        if it.next() == Some("\t}".to_owned()) {
            return Ok(None);
        }
        ret.name = it.next().unwrap().get(3..).unwrap().to_owned();
        ret.plural = it.next().unwrap().get(3..).unwrap().to_owned();
        ret.adjective = it.next().unwrap().get(3..).unwrap().to_owned();
        ret.portrait = it.next().unwrap().get(3..).unwrap().to_owned();
        it.next();
        while let Some(line) = it.next() {
            match line.get(..).unwrap() {
                "\t\t\t}" => break,
                _ => ret.traits.push(line.get(4..).unwrap().to_owned()),
            }
        }
        it.next();
        Ok(Some(ret))
    }
    fn save(&self, id: usize, save: &mut File) -> io::Result<()> {
        save.write_all(
            format!(
                "\t\t{} {{\n\t\t\t{}\n\t\t\t{}\n\t\t\t{}\n\t\t\t{}\n\t\t\ttraits {{\n",
                id, self.name, self.plural, self.adjective, self.portrait
            )
            .as_bytes(),
        )?;
        for i in self.traits.iter() {
            save.write_all(format!("\t\t\t\t{}\n", i).as_bytes())?;
        }
        save.write_all(b"\t\t\t}\n\t\t}\n")?;
        Ok(())
    }
    ///takes a string in the format of a stellaris save, extracting the species name.
    fn set_name(&mut self, inp: &str) {
        self.name = format!("{}", inp.get(8..(inp.chars().count() - 1)).unwrap());
    }
    ///takes a string in the format of a stellaris save, extracting the species plural.
    fn set_plural(&mut self, inp: &str) {
        self.plural = format!("{}", inp.get(10..(inp.chars().count() - 1)).unwrap());
    }
    ///takes a string in the format of a stellaris save, extracting the species adjective.
    fn set_adjective(&mut self, inp: &str) {
        self.adjective = format!("{}", inp.get(13..(inp.chars().count() - 1)).unwrap());
    }
    ///takes a string in the format of a stellaris save, extracting the species portrait.
    fn set_portrait(&mut self, inp: &str) {
        self.portrait = format!("{}", inp.get(12..(inp.chars().count() - 1)).unwrap());
    }
    ///takes an iterator of gamestate at the correct position,
    /// extracting species traits in the form of strings.
    fn set_traits<'a>(&mut self, inp: &mut impl Iterator<Item = &'a str>) {
        while let Some(line) = inp.next() {
            if line == "\t\t}" {
                break;
            }
            self.traits.push(format!(
                "{}",
                line.get(10..(line.chars().count() - 1)).unwrap()
            ));
        }
    }
}

#[derive(Debug, Clone)]
pub struct GalObject {
    id: usize,
    x: i64,
    y: i64,
    typ: String,
    name: String,
    planets: Vec<usize>,
}

impl GalObject {
    pub fn gx(&self) -> f64 {
        self.x as f64 / 1000.0
    }
    pub fn gy(&self) -> f64 {
        self.y as f64 / 1000.0
    }
    fn new<'a>(it: &mut impl Iterator<Item = &'a str>) -> Result<Option<GalObject>, String> {
        let mut ret = GalObject {
            id: std::usize::MAX,
            x: std::i64::MAX,
            y: std::i64::MAX,
            typ: String::new(),
            name: String::new(),
            planets: Vec::new(),
        };
        ret.id = match it.next() {
            Some("}") => return Ok(None),
            Some(line) => match line.get(1..(line.chars().count() - 2)) {
                Some(a) => parser(a)?,
                None => return Err(format!("Couldn't parse galactic object ID: >{}<", line)),
            },
            None => {
                return Err(format!(
                    "Couldn't parse galactic object ID; end of iterator"
                ))
            }
        };
        while let Some(line) = it.next() {
            if line == "\t}" {
                break;
            }
            match line.get(..5) {
                Some("\t\t\tx=") => {
                    ret.x = match line.get(5..line.chars().count()) {
                        Some(a) => str_to_coord(a)?,
                        None => {
                            return Err(format!(
                                "Couldn't get x-coordinate of line >{}< for GalObj >{}<",
                                line, ret.id
                            ))
                        }
                    }
                }
                Some("\t\t\ty=") => {
                    ret.y = match line.get(5..line.chars().count()) {
                        Some(a) => str_to_coord(a)?,
                        None => {
                            return Err(format!(
                                "Couldn't get y-coordinate of line >{}< for GalObj >{}<",
                                line, ret.id
                            ))
                        }
                    }
                }
                Some("\t\ttyp") => {
                    ret.typ = match line.get(7..line.chars().count()) {
                        Some(a) => a.to_owned(),
                        None => {
                            return Err(format!(
                                "Couldn't get type of line >{}< for GalObj >{}<",
                                line, ret.id
                            ))
                        }
                    }
                }
                Some("\t\tnam") => {
                    ret.name = match line.get(8..(line.chars().count() - 1)) {
                        Some(a) => a.to_owned(),
                        None => {
                            return Err(format!(
                                "Couldn't get name of line >{}< for GalObj >{}<",
                                line, ret.id
                            ))
                        }
                    }
                }
                Some("\t\tpla") => ret.planets.push(match line.get(9..line.chars().count()) {
                    Some(a) => parser(a)?,
                    None => {
                        return Err(format!(
                            "Couldn't get name of line >{}< for GalObj >{}<",
                            line, ret.id
                        ))
                    }
                }),
                None => {
                    if line == "\t}" {
                        break;
                    }
                }
                _ => {}
            }
        }
        Ok(Some(ret))
    }
    fn newe() -> GalObject {
        GalObject {
            id: std::usize::MAX,
            x: std::i64::MAX,
            y: std::i64::MAX,
            typ: String::new(),
            name: String::new(),
            planets: Vec::new(),
        }
    }
    fn read(it: &mut impl Iterator<Item = String>) -> io::Result<Option<GalObject>> {
        let mut ret = GalObject::newe();
        ret.id = match it.next().unwrap().get(..) {
            Some("\t}") => return Ok(None),
            Some(a) => parser(
                a.get(2..(a.chars().count() - 2))
                    .expect("Error in planet id , planet"),
            )
            .expect("Test"),
            None => panic!("fuck"),
        };
        if let Some(a) = it.next() {
            ret.name = a.get(3..a.chars().count()).unwrap().to_owned();
        }
        if let Some(a) = it.next() {
            ret.typ = a.get(3..a.chars().count()).unwrap().to_owned();
        }
        let temp = match it.next() {
            Some(a) => a,
            None => panic!("no."),
        };
        let mut temp = temp.get(4..(temp.chars().count() - 1)).unwrap().split(", ");
        ret.x = temp.next().unwrap().parse().unwrap();
        ret.y = temp.next().unwrap().parse().unwrap();
        it.next();
        loop {
            match it.next().unwrap().get(..) {
                Some("\t\t\t}") => break,
                Some(a) => ret.planets.push(a.get(4..).unwrap().parse().unwrap()),
                _ => panic!("NO"),
            }
        }
        it.next();
        Ok(Some(ret))
    }
    fn save(&self, save: &mut File) -> io::Result<()> {
        save.write_all(
            format!(
                "\t\t{} {{\n\t\t\t{}\n\t\t\t{}\n\t\t\t({}, {})\n",
                self.id, self.name, self.typ, self.x, self.y
            )
            .as_bytes(),
        )?;
        save.write_all(b"\t\t\tplanets {\n")?;
        for i in self.planets.iter() {
            save.write_all(format!("\t\t\t\t{}\n", i).as_bytes())?;
        }
        save.write_all(b"\t\t\t}\n\t\t}\n")?;
        Ok(())
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

///reads coordinates on the form "140.540" to i64, going via f64 and multiplying by 1000. Returns an Err if parsing fails.
fn str_to_coord(inp: &str) -> Result<i64, String> {
    match inp.parse::<f64>() {
        Ok(a) => Ok((1000f64 * a) as i64),
        Err(e) => Err(format!("Couldn't parse >{}<: error message >{}<", inp, e)),
    }
}

fn str_to_color(inp: &str) -> Option<String> {
    match inp {
        "null" => None,
        _ => Some(inp.to_owned()),
    }
}

///reads all .sav files in the relevant location,
/// extracting gamestate into strings for future analysis
fn get_gamestates() -> Result<Vec<String>, String> {
    let mut ret = Vec::new();
    match glob("input/*.sav") {
        Ok(paths) => {
            for potential_path in paths {
                match potential_path {
                    Ok(path) => match open_zip(path) {
                        Ok(content) => ret.push(content),
                        Err(e) => return Err(format!("Cannot open gamestate: {}", e)),
                    },
                    Err(e) => return Err(format!("Cannot retrieve PathBuf from glob: {}", e)),
                }
            }
        }
        Err(e) => return Err(format!("Cannot glob: {}", e)),
    }
    Ok(ret)
}

///opens a zip file from the path (PathBuf), returning a result
/// with either the gamestate content or an error
fn open_zip(filename: PathBuf) -> zip::result::ZipResult<String> {
    let mut temp = String::new();
    let mut zip = zip::ZipArchive::new(File::open(filename)?)?;
    zip.by_name("gamestate")?.read_to_string(&mut temp)?;
    Ok(temp)
}

///turns &str "yes" and "no" to bool false and true, returning an error if neither
fn str_to_bool(inp: &str) -> Result<bool, String> {
    match inp {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ => Err(format!("Couldn't turn >{}< to bool", inp)),
    }
}

///parses a &str to a number, changing the Result-type.
fn parser(inp: &str) -> Result<usize, String> {
    match inp.parse() {
        Ok(a) => Ok(a),
        Err(e) => Err(format!(
            "Error parsing value >{}< with error code >{}<",
            inp, e
        )),
    }
}

///takes an iterator over the file, reading everything between "pop={ ... }",
/// returning a hashmap of usize(planet id) to a vector of pops (inhabitants),
/// wrapped in a result.
fn pop_analyser<'a>(
    it: &mut impl Iterator<Item = &'a str>,
) -> Result<HashMap<usize, Vec<Pop>>, String> {
    let mut ret = HashMap::new();
    while let Some(line) = it.next() {
        if line == "}" {
            break;
        }
        let (id, temp) = Pop::new(
            it,
            match line.get(1..(line.chars().count() - 2)) {
                Some(a) => parser(a)?,
                None => return Err(format!("Couldn't get species id from: >{}<", line)),
            },
        )?;
        ret.entry(id).or_insert(Vec::new()).push(temp);
    }
    Ok(ret)
}

fn gal_obj_analyser<'a>(it: &mut impl Iterator<Item = &'a str>) -> Result<Vec<GalObject>, String> {
    let mut ret = Vec::new();
    while let Some(a) = GalObject::new(it)? {
        ret.push(a);
    }
    Ok(ret)
}

fn planet_analyser<'a>(
    it: &mut impl Iterator<Item = &'a str>,
    pops: &mut HashMap<usize, Vec<Pop>>,
) -> Result<HashMap<usize, Planet>, String> {
    let mut ret = HashMap::new();
    while let Some(plan) = Planet::new(it, pops)? {
        if plan.1 {
            ret.insert(plan.0.id, plan.0);
        }
    }
    Ok(ret)
}

fn empire_analyser<'a>(
    it: &mut impl Iterator<Item = &'a str>,
    inp: &mut Galaxy,
    plan: &mut HashMap<usize, Planet>,
) -> Result<(), Box<dyn Error>> {
    while let Some(a) = Empire::new(it, plan)? {
        inp.push(a);
    }
    Ok(())
}

///takes the contents of gamestate in a .sav file, returning a Galaxy packed in a result
fn save_analyser(file: &str, cmp: &mut Everything) -> Result<Option<Galaxy>, Box<dyn Error>> {
    let mut ret = Galaxy::new();
    let mut it = file.split('\n').skip(3);
    match it.next() {
        Some(line) => ret.setdate(line).expect("Sorry"),
        None => panic!("Error getting next line at: Get_Date"),
    };
    if cmp.times.contains_key(&ret.date) {
        return Ok(None);
    }
    let mut pops = HashMap::new();
    let mut planets = HashMap::new();
    let mut temp = Vec::new();
    while let Some(line1) = it.next() {
        match line1 {
            "species={" => ret.get_species(&mut it),
            "pop={" => pops = pop_analyser(&mut it)?,
            "galactic_object={" => temp = gal_obj_analyser(&mut it)?,
            "\tplanet={" => {
                planets = planet_analyser(&mut it, &mut pops)?;
                for plan in temp.iter_mut() {
                    plan.planets.retain(|x| planets.get(x).is_some());
                }
            }
            "country={" => empire_analyser(&mut it, &mut ret, &mut planets)?,
            _ => {}
        }
    }
    ret.push_g(temp);
    Ok(Some(ret))
}

fn read() -> Result<Everything, Box<dyn Error>> {
    Ok(Everything::read(
        &mut BufReader::new(File::open("save.txt")?)
            .lines()
            .map(|x| x.expect("Error in BufReader")),
    )?)
}

impl fmt::Debug for Planet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\t{}\t{}\t{}\n",
            self.id, self.name, self.typ, self.size
        )?;
        for i in self.population.iter() {
            write!(f, "{:?}\n", i)?;
        }
        write!(f, "")
    }
}
impl fmt::Debug for Pop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},\t{},\t{},\t{},\t{},\t{}",
            self.id, self.species, self.ethic, self.job, self.category, self.slave
        )
    }
}
impl fmt::Debug for Empire {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},\t{},\t{},\t{},\t\t{:?},\t{:?}",
            self.id,
            self.name,
            self.adjective,
            self.planets.len(),
            self.ethics,
            self.color
        )
    }
}
