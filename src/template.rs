#[derive(Clone, Debug, Default)]
pub struct Template {
    name: String,
    is_selected: bool,
}

impl Template {
    fn new(name: String) -> Self {
        Self {
            name: name.to_owned(),
            is_selected: false,
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn is_selected(&self) -> bool {
        self.is_selected
    }
}

#[derive(Clone, Debug, Default)]
pub struct Templates {
    options: Vec<Template>,
}

impl Templates {
    pub(crate) fn new() -> Self {
        Self { options: vec![] }
    }
    pub(crate) fn set_list(&mut self, names: Vec<String>) {
        self.options.clear();
        for name in names {
            self.options.push(Template::new(name));
        }
    }
    pub(crate) fn select_template(&mut self, name: &str) {
        self.set_template_selected_flag(name, true);
    }
    pub(crate) fn unselect_template(&mut self, name: &str) {
        self.set_template_selected_flag(name, false);
    }
    pub(crate) fn selected_templates(&self) -> Vec<Template> {
        self.get_list(true)
    }
    pub(crate) fn unselected_templates(&self) -> Vec<Template> {
        self.get_list(false)
    }
    pub(crate) fn selected_template_names(&self) -> Vec<String> {
        self.selected_templates()
            .iter()
            .map(|template| template.name().to_string())
            .collect()
    }
    pub(crate) fn any_selected(&self) -> bool {
        self.options.iter().any(|t| t.is_selected())
    }
}
impl Templates {
    fn set_template_selected_flag(&mut self, name: &str, selected: bool) {
        self.options.iter_mut().for_each(|option| {
            if option.name == name {
                option.is_selected = selected;
            }
        });
    }
    fn get_list(&self, selected: bool) -> Vec<Template> {
        self.options
            .iter()
            .filter(|option| option.is_selected() == selected)
            .cloned()
            .collect()
    }
}
