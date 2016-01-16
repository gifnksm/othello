use std::marker::PhantomData;

use conrod::DropDownList;

#[derive(Clone, Debug)]
pub struct DdlBuilder<T> {
    strings: Vec<String>,
    selected_idx: Option<usize>,
    phantom: PhantomData<T>,
}

impl<T> DdlBuilder<T> {
    pub fn new() -> DdlBuilder<T>
        where T: DdlString + Default
    {
        let strings = T::create_strings();
        let default_str = T::default().to_ddl_string();
        let selected = strings.iter().position(|x| x == &default_str);
        DdlBuilder {
            strings: strings,
            selected_idx: selected,
            phantom: PhantomData,
        }
    }

    pub fn build_drop_down_list<F>(&mut self) -> DropDownList<F>
        where T: DdlString
    {
        DropDownList::new(&mut self.strings, &mut self.selected_idx)
    }
}

pub trait DdlString: Sized {
    fn from_ddl_str(s: &str) -> Option<Self>;
    fn to_ddl_string(&self) -> String;
    fn create_strings() -> Vec<String>;
}