use ievr_cfg_bin_editor_core::Row;

use crate::common::{Element, parse_int_value};

pub struct Aura {
    pub(crate) aura_id: i32,
    pub(crate) name_id: i32,
    pub(crate) skill_id: Option<i32>,
    pub(crate) element: Element
}

impl From<&Row> for Aura {
    fn from(row: &Row) -> Self {
        Aura {
            aura_id: parse_int_value(&row.values[0][0]),
            name_id: parse_int_value(&row.values[2][0]),
            skill_id: {
                let value = parse_int_value(&row.values[6][0]);
                if value != 0 { Some(value) } else { None }
            },
            element: Element::from(parse_int_value(&row.values[8][0]))
        }
    }
}