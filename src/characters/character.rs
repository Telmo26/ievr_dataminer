#[derive(Debug)]
pub struct Character {
    pub index: i32,
    pub name_id: i32,
    pub element: Element,
    pub main_position: Position,
    pub alt_position: Position,
    pub style: Style,
    pub lvl50_stats: Stats,
    pub lvl99_stats: Stats,
    pub series_id: i32,
    
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Element {
    WIND = 1,
    FOREST = 2,
    FIRE = 3,
    MOUNTAIN = 4,
    UNKNOWN = 5,
}

impl From<i32> for Element {
    fn from(value: i32) -> Self {
        match value {
            1 => Element::WIND,
            2 => Element::FOREST,
            3 => Element::FIRE,
            4 => Element::MOUNTAIN,
            _ => Element::UNKNOWN,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    GK = 1,
    DF = 4,
    MF = 3,
    FW = 2,
    UNKNOWN = 5,
}

impl From<i32> for Position {
    fn from(value: i32) -> Self {
        match value {
            1 => Position::GK,
            2 => Position::FW,
            3 => Position::MF,
            4 => Position::DF,
            _ => Position::UNKNOWN,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Style {
    BREACH = 0,
    COUNTER = 1,
    BOND = 2,
    TENSION = 3,
    ROUGH = 4,
    JUSTICE = 5,
    UNKNOWN = 6,
}

impl From<i32> for Style {
    fn from(value: i32) -> Self {
        match value {
            0 => Style::BREACH,
            1 => Style::COUNTER,
            2 => Style::BOND,
            3 => Style::TENSION,
            4 => Style::ROUGH,
            5 => Style::JUSTICE,
            _ => Style::UNKNOWN
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Stats {
    pub kick: u16,
    pub control: u16,
    pub technique: u16,
    pub pressure: u16,
    pub physical: u16,
    pub agility: u16,
    pub intelligence: u16,
}