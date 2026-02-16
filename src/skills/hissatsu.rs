use ievr_cfg_bin_editor_core::Row;

use crate::common::{Element, parse_byte_value, parse_f32_value, parse_int_value, parse_short_value, parse_uint_value};

pub struct Hissatsu {
    pub(crate) skill_id: i32,
    pub(crate) name_id: i32,
    pub(crate) power: i16,
    pub(crate) element: Element,
    pub(crate) category: Category,
    pub(crate) growth_rate: u8,
    pub(crate) is_block: bool,
    pub(crate) is_longshot: bool,
    pub(crate) tp_consumption: u8,
    pub(crate) cooldown: f32,
}

impl From<&Row> for Hissatsu {
    fn from(row: &Row) -> Self {
        
        let skill_effect_bit_flag = parse_int_value(&row.values[9][0]);

        let is_block = (skill_effect_bit_flag & 16) != 0;
        let is_longshot = (skill_effect_bit_flag & 4) != 0;

        Hissatsu {
            skill_id: parse_uint_value(&row.values[0][0]).cast_signed(),
            name_id: parse_uint_value(&row.values[6][0]).cast_signed(),
            power: parse_short_value(&row.values[10][0]),
            element: (parse_byte_value(&row.values[12][0]) as i32).into(),
            category: Category::from(parse_byte_value(&row.values[14][0])),
            growth_rate: parse_byte_value(&row.values[15][0]),
            is_block,
            is_longshot,
            tp_consumption: parse_byte_value(&row.values[17][0]),
            cooldown: parse_f32_value(&row.values[19][0]),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Category {
    Unknown = 0,
    Shoot = 1,
    Dribble = 2,
    Defense = 3,
    Goalkeeper = 4,
    Passive = 6,
    StatsIncrease = 9
}

impl From<u8> for Category {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Shoot,
            2 => Self::Dribble,
            3 => Self::Defense,
            4 => Self::Goalkeeper,
            6 => Self::Passive,
            9 => Self::StatsIncrease,
            _ => Self::Unknown,
        }
    }
}
