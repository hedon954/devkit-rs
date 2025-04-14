use std::collections::HashMap;

#[allow(unused)]
fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
    let mut map = HashMap::new();
    for (index, num) in nums.iter().enumerate() {
        match map.get(&(target - num)) {
            Some(i) => {
                return vec![*i, index as i32];
            }
            _ => {
                map.insert(num, index as i32);
            }
        }
    }
    panic!("No solution found");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_two_sum() {
        let nums = vec![2, 7, 11, 15];
        let target = 9;
        assert_eq!(two_sum(nums, target), vec![0, 1]);
    }
}
