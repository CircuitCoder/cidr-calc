#[derive(Debug, Eq, PartialEq)]
pub struct V4(pub u32, pub u8);

#[derive(Debug, Eq, PartialEq)]
pub struct V6(pub u128, pub u8);

impl ToString for V4 {
    fn to_string(&self) -> String {
        // Mask end zeros
        if self.1 == 0 {
            return "0.0.0.0/0".to_owned();
        }

        let masked = self.0 & !((1u32 << (32 - self.1)) - 1);
        format!("{}/{}", masked.to_be_bytes().map(|e| e.to_string()).join("."), self.1)
    }
}

impl ToString for V6 {
    fn to_string(&self) -> String {
        if self.1 == 0 {
            return "::/0".to_owned();
        }

        let masked = self.0 & !((1u128 << (128 - self.1)) - 1);
        let mut segs = <[Option::<String>; 8]>::default();

        for grpidx in 0..8 {
            let grp = (masked >> ((8 - grpidx - 1) * 16)) & 0xFFFFu128;
            if grp != 0 {
                segs[grpidx] = Some(format!("{:x}", grp));
            }
        }

        let mut zero_lengths = [0; 8];
        zero_lengths[0] = if segs[0].is_none() { 1 } else { 0 };
        let mut zero_lengths_max = (zero_lengths[0], 0usize);
        for grpidx in 1..8 {
            zero_lengths[grpidx] = if segs[grpidx].is_none() {
                zero_lengths[grpidx - 1] + 1
            } else {
                0
            };

            if zero_lengths_max.0 < zero_lengths[grpidx] {
                zero_lengths_max = (zero_lengths[grpidx], grpidx);
            }
        }

        // Format
        if zero_lengths_max.0 == 0 {
            // No zero segments
            format!("{}/{}", segs.map(Option::unwrap).join(":"), self.1)
        } else {
            let seg_start = zero_lengths_max.1 + 1 - zero_lengths_max.0;
            let seg_head = &segs[0..seg_start];
            let seg_tail= &segs[(zero_lengths_max.1 + 1)..8];
            let head = seg_head.iter().map(|e| e.as_ref().map(|i| i.as_str()).unwrap_or("0")).collect::<Vec<_>>().join(":");
            let tail = seg_tail.iter().map(|e| e.as_ref().map(|i| i.as_str()).unwrap_or("0")).collect::<Vec<_>>().join(":");
            format!("{}::{}/{}", head, tail, self.1)
        }
    }
}