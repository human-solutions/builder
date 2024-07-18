/// A struct that divides a string into parts using a set of dividers, it assumes that each provided divider appear only once and in the order they are provided.
///
/// Example:
/// ```
/// let str = "The X-ray machine is top-of-the-line."
/// let dividers = ["-", "/", "of"];
/// let mut divider = StrDivider::new(str, &dividers);
/// assert_eq!(divider.next(), Some("The X"));
/// assert_eq!(divider.next(), Some("-"));
/// assert_eq!(divider.next(), Some("ray machine is top-"));
/// assert_eq!(divider.next(), Some("of"));
/// assert_eq!(divider.next(), Some("-the-line."));
/// ```
pub(super) struct StrDivider<'a> {
    string: &'a str,
    dividers: &'a [&'a str],
    div_index: usize,
}

impl<'a> StrDivider<'a> {
    /// Create a new `StrDivider` instance.
    pub fn new(s: &'a str, dividers: &'a [&'a str]) -> Self {
        Self {
            string: s,
            dividers,
            div_index: 0,
        }
    }

    /// Get the next part of the string.
    fn next_inner(&mut self) -> Option<&'a str> {
        if let Some(divider) = self.dividers.get(self.div_index) {
            if let Some(pos) = self.string.find(divider) {
                if pos == 0 {
                    self.div_index += 1;
                    self.string = self.string.get(pos + divider.len()..)?;

                    return Some(divider);
                }

                let (part, rest) = self.string.split_at(pos);
                self.string = rest;

                return Some(part);
            }

            if !self.string.is_empty() {
                self.div_index += 1;
                return self.next_inner();
            }
        }

        if !self.string.is_empty() {
            let rest = self.string;
            self.string = "";

            return Some(rest);
        }

        None
    }
}

impl<'a> Iterator for StrDivider<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_inner()
    }
}

#[test]
fn str_divider() {
    let str = "The X-ray machine is top-of-the-line.";
    let dividers = ["-", "/", "of"];
    let mut divider = StrDivider::new(str, &dividers);

    assert_eq!(divider.next(), Some("The X"));
    assert_eq!(divider.next(), Some("-"));
    assert_eq!(divider.next(), Some("ray machine is top-"));
    assert_eq!(divider.next(), Some("of"));
    assert_eq!(divider.next(), Some("-the-line."));
}
