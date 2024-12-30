#[derive(Debug, Default)]
pub struct Stack<T> {
    size: usize,
    data: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self {
            size: 0,
            data: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            size: 0,
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn clear(&mut self) {
        self.size = 0;
        self.data.clear();
    }

    pub fn push(&mut self, item: T) {
        self.data.push(item);
        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        self.size -= 1;
        self.data.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        self.data.get(self.size - 1)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            return None;
        }
        self.data.get_mut(self.size - 1)
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            stack: self.data.iter().collect(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            stack: self.data.iter_mut().collect(),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

pub struct IntoIter<T>(Stack<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

pub struct Iter<'a, T> {
    stack: Vec<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop()
    }
}

pub struct IterMut<'a, T> {
    stack: Vec<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack() {
        let stack: Stack<i32> = Stack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_with_capacity() {
        let stack: Stack<i32> = Stack::with_capacity(10);
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert!(stack.data.capacity() >= 10);
    }

    #[test]
    fn test_push_pop() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_peek() {
        let mut stack = Stack::new();
        assert_eq!(stack.peek(), None);

        stack.push(1);
        stack.push(2);
        assert_eq!(stack.peek(), Some(&2));

        // Ensure peek doesn't remove the element
        assert_eq!(stack.peek(), Some(&2));
        assert_eq!(stack.len(), 2);
    }

    #[test]
    fn test_peek_mut() {
        let mut stack = Stack::new();
        assert_eq!(stack.peek_mut(), None);

        stack.push(1);
        stack.push(2);

        if let Some(value) = stack.peek_mut() {
            *value = 3;
        }

        assert_eq!(stack.peek(), Some(&3));
    }

    #[test]
    fn test_clear() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        stack.clear();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_iter() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        let mut iter = stack.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), None);

        // Original stack should remain unchanged
        assert_eq!(stack.len(), 3);
    }

    #[test]
    fn test_iter_mut() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        for item in stack.iter_mut() {
            *item *= 2;
        }

        assert_eq!(stack.pop(), Some(6));
        assert_eq!(stack.pop(), Some(4));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_into_iter() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        let collected: Vec<i32> = stack.into_iter().collect();
        assert_eq!(collected, vec![3, 2, 1]);
    }
}
