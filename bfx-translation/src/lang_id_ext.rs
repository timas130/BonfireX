use unic_langid::LanguageIdentifier;

pub trait LanguageIdentifierExt {
    fn only_language(&self) -> LanguageIdentifier;

    fn only_script(&self) -> LanguageIdentifier;

    fn only_region(&self) -> LanguageIdentifier;
}

impl LanguageIdentifierExt for LanguageIdentifier {
    fn only_language(&self) -> LanguageIdentifier {
        Self::from_parts(self.language, None, None, &[])
    }

    fn only_script(&self) -> LanguageIdentifier {
        Self::from_parts(self.language, self.script, None, &[])
    }

    fn only_region(&self) -> LanguageIdentifier {
        Self::from_parts(self.language, self.script, self.region, &[])
    }
}
