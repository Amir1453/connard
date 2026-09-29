#[derive(Clone, PartialEq)]
pub struct Stack<T> {
    stack: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn new_with(t: T) -> Self {
        Self { stack: vec![t] }
    }

    pub fn push(&mut self, t: T) {
        self.stack.push(t);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }

    pub fn top(&self) -> Option<&T> {
        self.stack.last()
    }

    pub fn top_mut(&mut self) -> Option<&mut T> {
        self.stack.last_mut()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.stack.iter()
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self {
            stack: Default::default(),
        }
    }
}
