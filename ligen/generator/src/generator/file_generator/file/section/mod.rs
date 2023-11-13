use crate::prelude::*;
use std::borrow::Borrow;

pub mod content;
pub mod template;

pub use content::*;
use is_tree::*;
pub use template::*;

use std::ops::Range;

#[derive(Debug)]
pub struct FileSection {
    /// File section name.
    pub name: String,
    /// File section content.
    pub content: Vec<Box<dyn FileSectionContent>>
}

impl From<String> for FileSection {
    fn from(name: String) -> Self {
        let content = Default::default();
        Self { name, content }
    }
}

impl FileSection {
    /// Creates a new FileSection.
    pub fn new(name: impl Into<String>) -> Self {
        Self::from(name.into())
    }

    /// Creates a new FileSection from a template.
    pub fn from_template(template: &SectionTemplate) -> Result<Self> {
        let mut section = Self::new(&template.name);
        let sections = Self::get_sections_ranges(template)?;
        section.write_from_template(template, sections)?;

        Ok(section)
    }

    /// Writes the content to the file section at the specified index.
    pub fn indexed_write<S: Into<String>>(&mut self, index: usize, content: S) {
        self.content.insert(index, Box::new(content.into()))
    }

    /// Writes the content to the file section at the specified index and adds a new line.
    pub fn indexed_writeln<S: Into<String>>(&mut self, index: usize, content: S) {
        let mut string = content.into();
        string.push('\n');
        self.indexed_write(index, string);
    }

    /// Writes the content to the file buffer.
    pub fn write<S: Into<String>>(&mut self, content: S) {
        self.content.push(Box::new(content.into()));
    }

    /// Writes the content to the file buffer and adds a new line.
    pub fn writeln<S: Into<String>>(&mut self, content: S) {
        let mut string = content.into();
        string.push('\n');
        self.content.push(Box::new(string));
    }    
}

impl HasIdentifier for FileSection {
    type Identifier = String;
    fn identifier(&self) -> &Self::Identifier {
        &self.name
    }
}

impl IsTree for FileSection {
    fn add_branch(&mut self, section: impl Into<Self>) -> &mut Self where Self: Sized {
        self.content.push(Box::new(section.into()));
        self
            .content
            .last_mut()
            .unwrap()
            .as_section_mut()
            .unwrap()
    }

    fn get<K>(&self, key: K) -> Option<&Self>
    where K: Into<Self::Identifier>, Self::Identifier: Borrow<Self::Identifier>
    {
        let name = key.into();
        let name = name.borrow();
        self.content
            .iter()
            .find_map(|content| {
                content
                    .as_section()
                    .and_then(|section|
                        if section.name == name {
                            Some(section)
                        } else {
                            None
                        }
                    )
            })
    }

    fn get_mut<K>(&mut self, key: K) -> Option<&mut Self>
    where K: Into<Self::Identifier>, Self::Identifier: std::borrow::BorrowMut<Self::Identifier>
    {
        let name = key.into();
        let name = name.borrow();
        self.content
            .iter_mut()
            .find_map(|content| {
                content
                    .as_section_mut()
                    .and_then(|section|
                        if section.name == name {
                            Some(section)
                        } else {
                            None
                        }
                    )
            })
    }

    fn branches<'a>(&'a self) -> Box<dyn Iterator<Item = &Self> + 'a> {
        Box::new(self.content.iter().filter_map(|content| content.as_section()))
    }

    fn branches_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Self> + 'a> {
        Box::new(self.content.iter_mut().filter_map(|content| content.as_section_mut()))
    }
}

impl FileSection {
    /// Section start.
    const SECTION_START: &'static str = "[section(";
    /// Section end.
    const SECTION_END: &'static str = ")]";

    /// Gets the sections ranges.
    fn get_sections_ranges(template: &SectionTemplate) -> Result<Vec<Range<usize>>> {
        let template = template.content.as_str();
        let mut sections = Vec::new();
        for (index, _) in template.match_indices(Self::SECTION_START) {
            let index_start = index;
            let index_end = template[index_start..]
                .find(Self::SECTION_END)
                .ok_or_else(|| Error::Message("Failed to parse template: missing section end.".to_string()))?;
            let index_end = index_end + index_start + Self::SECTION_END.len();
            sections.push(index_start..index_end);
        }
        Ok(sections)        
    }

    /// Registers the sections and writes the text in-between them. See the `template_with_content` test.
    fn write_from_template(&mut self, template: &SectionTemplate, sections: impl IntoIterator<Item = Range<usize>>) -> Result<()> {
        let mut start = 0;
        for section in sections {
            let before = &template.content[start..section.start];
            if !before.is_empty() {
                self.write(before);
            }
            start = section.end;
            let section = &template.content[(section.start + Self::SECTION_START.len())..(section.end - Self::SECTION_END.len())];
            let section = if let Some(template) = template.get(section) {
                FileSection::from_template(template)?
            } else {
                FileSection::new(section)
            };
            self.add_branch(section);
        }
        let after = &template.content[start..];
        if !after.is_empty() {
            self.write(after);
        }
        Ok(())
    }
}

impl std::fmt::Display for FileSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for file_content in &self.content {
            write!(f, "{}", file_content)?;
        }
        Ok(())
    }
}
