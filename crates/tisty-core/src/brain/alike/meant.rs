#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Meant {
    scale: f32,
    values: Vec<i8>,
}

impl Meant {
    pub(crate) fn of(vector: &[f32]) -> Self {
        let most = vector.iter().fold(0f32, |most, one| most.max(one.abs()));
        let scale = most / 127.0;
        let values = vector
            .iter()
            .map(|one| match scale > 0.0 {
                true => (one / scale).round().clamp(-127.0, 127.0) as i8,
                false => 0,
            })
            .collect();
        Self { scale, values }
    }

    pub(crate) fn read(bytes: &[u8]) -> Option<Self> {
        let (scale, values) = bytes.split_first_chunk::<4>()?;
        Some(Self {
            scale: f32::from_le_bytes(*scale),
            values: values.iter().map(|one| i8::from_le_bytes([*one])).collect(),
        })
    }

    pub(crate) fn said(&self) -> Vec<u8> {
        let mut bytes = self.scale.to_le_bytes().to_vec();
        bytes.extend(self.values.iter().map(|one| one.to_le_bytes()[0]));
        bytes
    }

    pub(crate) fn scale(&self) -> f32 {
        self.scale
    }

    pub(crate) fn values(&self) -> &[i8] {
        &self.values
    }

    pub(crate) fn says_something(&self) -> bool {
        self.scale > 0.0 && !self.values.is_empty()
    }
}

#[cfg(test)]
#[path = "meant_test.rs"]
mod tests;
