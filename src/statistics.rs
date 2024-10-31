use std::iter::Sum;

pub fn mean<'a, T>(numbers: &'a [T]) -> Option<f64>
where
    T: Into<f64> + Sum<&'a T>,
{
    let sum = numbers.iter().sum::<T>();
    let length = numbers.len() as f64;

    match length {
        positive if positive > 0_f64 => Some(sum.into() / length),
        _ => None,
    }
}

pub fn std_deviation<'a, T>(data: &'a [T]) -> Option<f64>
where
    T: Into<f64> + Sum<&'a T> + Copy,
{
    match (mean(data), data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let count: f64 = count as f64;
            let variance: f64 = data
                .iter()
                .map::<f64, _>(|value| {
                    let value: f64 = (*value).into();
                    let diff: f64 = data_mean - value;

                    diff * diff
                })
                .sum::<f64>()
                / count;

            Some(variance.sqrt())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_std_deviation() {
        let v = std_deviation(&[10, 12, 23, 23, 16, 23, 21, 16]);
        assert_eq!(v, Some(4.898979485566356));
    }

    #[test]
    fn test_mean() {
        let m = mean(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(m, Some(5.0));
    }
}
