//! CSS parsing with cssparser

/// Parsed stylesheet
#[derive(Debug, Default)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug)]
pub struct Rule {
    pub selectors: Vec<String>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug)]
pub struct Declaration {
    pub property: String,
    pub value: String,
}
