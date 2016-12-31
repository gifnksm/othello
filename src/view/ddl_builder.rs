use conrod::widget::DropDownList;
use conrod::widget::drop_down_list::Idx;
use std::marker::PhantomData;

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

    pub fn build_drop_down_list(&self) -> DropDownList<String> {
        DropDownList::new(&self.strings, self.selected_idx.clone())
    }
}

pub trait DdlString: Sized {
    fn from_ddl_index(i: Idx) -> Option<Self>;
    fn to_ddl_string(&self) -> String;
    fn create_strings() -> Vec<String>;
}
