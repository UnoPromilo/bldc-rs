use crate::RawInverterValues;

pub trait InverterOutput {
    fn max_duty(&self) -> u32;
    fn disable_outputs(&mut self);
    fn write_phase_duties(&mut self, duties: RawInverterValues);
    fn enable_outputs(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OutputState {
    Safe,
    Enabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InvalidDuty {
    pub requested: RawInverterValues,
    pub max_duty: u32,
}

pub struct SafeOutput<D: InverterOutput> {
    driver: D,
    state: OutputState,
}

impl<D: InverterOutput> SafeOutput<D> {
    pub fn new(mut driver: D) -> Self {
        driver.disable_outputs();
        driver.write_phase_duties(RawInverterValues::ZERO);
        Self {
            driver,
            state: OutputState::Safe,
        }
    }

    pub fn max_duty(&self) -> u32 {
        self.driver.max_duty()
    }

    pub fn state(&self) -> OutputState {
        self.state
    }

    pub fn enter_safe_state(&mut self) {
        if self.state == OutputState::Safe {
            return;
        }

        self.driver.disable_outputs();
        self.driver.write_phase_duties(RawInverterValues::ZERO);
        self.state = OutputState::Safe;
    }

    pub fn drive(&mut self, duties: RawInverterValues) -> Result<(), InvalidDuty> {
        let max_duty = self.driver.max_duty();
        if duties.u > max_duty || duties.v > max_duty || duties.w > max_duty {
            self.enter_safe_state();
            return Err(InvalidDuty {
                requested: duties,
                max_duty,
            });
        }

        self.driver.write_phase_duties(duties);
        if self.state == OutputState::Safe {
            self.driver.enable_outputs();
            self.state = OutputState::Enabled;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::vec::Vec;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Operation {
        Disable,
        Write(RawInverterValues),
        Enable,
    }

    struct FakeDriver {
        max_duty: u32,
        operations: Vec<Operation>,
    }

    impl FakeDriver {
        fn new(max_duty: u32) -> Self {
            Self {
                max_duty,
                operations: Vec::new(),
            }
        }
    }

    impl InverterOutput for FakeDriver {
        fn max_duty(&self) -> u32 {
            self.max_duty
        }

        fn disable_outputs(&mut self) {
            self.operations.push(Operation::Disable);
        }

        fn write_phase_duties(&mut self, duties: RawInverterValues) {
            self.operations.push(Operation::Write(duties));
        }

        fn enable_outputs(&mut self) {
            self.operations.push(Operation::Enable);
        }
    }

    #[test]
    fn startup_disables_outputs_and_writes_safe_duties() {
        let output = SafeOutput::new(FakeDriver::new(1000));

        assert_eq!(output.state(), OutputState::Safe);
        assert_eq!(
            output.driver.operations,
            [
                Operation::Disable,
                Operation::Write(RawInverterValues::ZERO)
            ]
        );
    }

    #[test]
    fn first_drive_writes_fresh_duties_before_enable() {
        let mut output = SafeOutput::new(FakeDriver::new(1000));
        output.driver.operations.clear();
        let duties = RawInverterValues {
            u: 100,
            v: 200,
            w: 300,
        };

        output.drive(duties).unwrap();

        assert_eq!(output.state(), OutputState::Enabled);
        assert_eq!(
            output.driver.operations,
            [Operation::Write(duties), Operation::Enable]
        );
    }

    #[test]
    fn repeated_drive_updates_without_reenabling() {
        let mut output = SafeOutput::new(FakeDriver::new(1000));
        output
            .drive(RawInverterValues {
                u: 100,
                v: 200,
                w: 300,
            })
            .unwrap();
        output.driver.operations.clear();
        let duties = RawInverterValues {
            u: 400,
            v: 500,
            w: 600,
        };

        output.drive(duties).unwrap();

        assert_eq!(output.driver.operations, [Operation::Write(duties)]);
    }

    #[test]
    fn invalid_duty_disables_before_writing_safe_duties() {
        let mut output = SafeOutput::new(FakeDriver::new(1000));
        output
            .drive(RawInverterValues {
                u: 100,
                v: 200,
                w: 300,
            })
            .unwrap();
        output.driver.operations.clear();

        let result = output.drive(RawInverterValues {
            u: 1001,
            v: 0,
            w: 0,
        });

        assert!(result.is_err());
        assert_eq!(output.state(), OutputState::Safe);
        assert_eq!(
            output.driver.operations,
            [
                Operation::Disable,
                Operation::Write(RawInverterValues::ZERO)
            ]
        );
    }

    #[test]
    fn drive_after_disable_writes_fresh_duties_before_reenable() {
        let mut output = SafeOutput::new(FakeDriver::new(1000));
        output
            .drive(RawInverterValues {
                u: 100,
                v: 200,
                w: 300,
            })
            .unwrap();
        output.enter_safe_state();
        output.driver.operations.clear();
        let duties = RawInverterValues {
            u: 300,
            v: 200,
            w: 100,
        };

        output.drive(duties).unwrap();

        assert_eq!(
            output.driver.operations,
            [Operation::Write(duties), Operation::Enable]
        );
    }
}
