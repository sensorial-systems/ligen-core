use std::fmt::Display;


#[derive(Clone)]
pub struct Path<'a, Segment>
where Segment: 'a
{
    pub segments: Vec<Segment>,
    phantom: std::marker::PhantomData<&'a Segment>
}

impl<'a, Segment> Path<'a, Segment> {
    pub fn join(&self, segment: impl Into<Segment>) -> Path<'a, Segment>
    where Path<'a, Segment>: Clone
    {
        let mut clone = self.clone();
        clone.segments.push(segment.into());
        clone
    }
}

impl<T> Default for Path<'_, T> {
    fn default() -> Self {
        let segments = Vec::new();
        let phantom = Default::default();
        Self { segments, phantom }
    }
}

impl<T> Path<'_, T> {
    pub fn new(&self) -> &T {
        self.segments.last().unwrap()
    }
}

impl<'a> From<&'a str> for Path<'a, &'a str> {
    fn from(value: &'a str) -> Path<'a, &'a str> {
        let segments = value.split("::").collect();
        let phantom = Default::default();
        Self { segments, phantom }
    }
}

impl<'a, Segment> From<Vec<Segment>> for Path<'a, Segment>
where Segment: 'a
{
    fn from(value: Vec<Segment>) -> Path<'a, Segment> {
        let segments = value;
        let phantom = Default::default();
        Path { segments, phantom }
    }
}

impl<'a, Segment> From<&'a [Segment]> for Path<'a, Segment>
where Segment: Copy
{
    fn from(value: &'a [Segment]) -> Path<'a, Segment> {
        let segments = value.to_vec();
        let phantom = Default::default();
        Path { segments, phantom }
    }
}

impl<'a> Display for Path<'a, String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.segments.join("::"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string() {
        let path = Path::from("A");
        assert_eq!(path.segments, ["A"]);
        let path = Path::from("A::B");
        assert_eq!(path.segments, ["A", "B"]);
    }

    #[test]
    fn from_array() {
        let array = ["A", "B", "C"];
        let slice = array.as_slice();
        let path: Path<'_, _> = Path::from(slice);
        assert_eq!(path.segments, ["A", "B", "C"]);
    }

    #[test]
    fn from_vector() {
        let vector = vec!["A", "B", "C"];
        let path = Path::from(vector);
        assert_eq!(path.segments, ["A", "B", "C"]);
    }
}
