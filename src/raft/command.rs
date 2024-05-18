pub trait Command<S> {
    fn apply(&self, state: &mut S);
}

impl<S> Command<S> for () {
    fn apply(&self, _state: &mut S) {}
}
