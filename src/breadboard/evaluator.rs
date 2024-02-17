use core::fmt::{self, Display, Write};

use uuid::{Uuid, uuid};

use crate::ftd_data::{SectionData, DataEntry};
use super::{BNumber, BQuaternion, BString, BVector3, Breadboard, Component, Line, LineInner, LineValue};

#[derive(Debug, Default)]
pub struct Evaluator {
    inputs: Vec<LineInner>,
    // there is one expression for each output
    exprs: Vec<EvaluatorExpression>,
}

impl Evaluator {
    /// Returns an expression representing the given input
    pub fn get_input(&mut self, input_line: LineInner) -> Option<EvaluatorExpression> {
        for (i, input) in self.inputs.iter().enumerate() {
            if input_line == *input {
                return EvaluatorExpression::from_input_index(i);
            }
        }

        let expr = EvaluatorExpression::from_input_index(self.inputs.len())?;
        self.inputs.push(input_line);
        Some(expr)
    }
}

impl Component for Evaluator {
    fn ftd_uuid(&self) -> Uuid {
        uuid!("7cf3b706-757e-428a-bb45-454a17ed710a")
    }

    fn section_data(&self) -> SectionData {
        let mut expr_string = String::new();
        for (i, expr) in self.exprs.iter().enumerate() {
            if i != 0 {
                expr_string.push_str(",");
            }

            write!(expr_string, "{expr}").unwrap();
        }

        SectionData::default()
            .with_entry(0, DataEntry::String(expr_string))
    }

    fn num_outputs(&self) -> usize {
        self.exprs.len()
    }

    fn inputs(&self) -> &[LineInner] {
        self.inputs.as_slice()
    }
}

macro_rules! make_bb_method {
    ($name:ident, $variant:ident, $in_name:ident: $in_type:ty, $out_type:ty) => {
        pub fn $name(&self, $in_name: Line<$in_type>) -> Line<$out_type> {
            self.evaluator_expr($in_name, EvaluatorExpression::$variant)
        }
    };

    ($name:ident, $variant:ident, $in_name1:ident: $in_type1:ty, $in_name2:ident: $in_type2:ty, $out_type:ty) => {
        pub fn $name(&self, $in_name1: Line<$in_type1>, $in_name2: Line<$in_type2>) -> Line<$out_type> {
            self.evaluator_expr2($in_name1, $in_name2, EvaluatorExpression::$variant)
        }
    };

    ($name:ident, $variant:ident, $in_name1:ident: $in_type1:ty, $in_name2:ident: $in_type2:ty, $in_name3:ident: $in_type3:ty, $out_type:ty) => {
        pub fn $name(&self, $in_name1: Line<$in_type1>, $in_name2: Line<$in_type2>, $in_name3: Line<$in_type3>) -> Line<$out_type> {
            self.evaluator_expr3($in_name1, $in_name2, $in_name3, EvaluatorExpression::$variant)
        }
    };
}

macro_rules! make_bb_method_named {
    ($name:ident, $variant:ident, $in_name:ident: $in_type:ty, $out_type:ty) => {
        pub fn $name(&self, $in_name: Line<$in_type>) -> Line<$out_type> {
            self.evaluator_expr($in_name, |a| EvaluatorExpression::$variant {
                $in_name: a,
            })
        }
    };

    ($name:ident, $variant:ident, $in_name1:ident: $in_type1:ty, $in_name2:ident: $in_type2:ty, $out_type:ty) => {
        pub fn $name(&self, $in_name1: Line<$in_type1>, $in_name2: Line<$in_type2>) -> Line<$out_type> {
            self.evaluator_expr2($in_name1, $in_name2, |a, b| EvaluatorExpression::$variant {
                $in_name1: a,
                $in_name2: b,
            })
        }
    };

    ($name:ident, $variant:ident, $in_name1:ident: $in_type1:ty, $in_name2:ident: $in_type2:ty, $in_name3:ident: $in_type3:ty, $out_type:ty) => {
        pub fn $name(&self, $in_name1: Line<$in_type1>, $in_name2: Line<$in_type2>, $in_name3: Line<$in_type3>) -> Line<$out_type> {
            self.evaluator_expr3($in_name1, $in_name2, $in_name3, |a, b, c| EvaluatorExpression::$variant {
                $in_name1: a,
                $in_name2: b,
                $in_name3: c,
            })
        }
    };
}

impl Breadboard {
    fn evaluator_expr<T: LineValue + ?Sized>(
        &self,
        val1: Line<impl LineValue + ?Sized>,
        expr_fn: impl FnOnce(Box<EvaluatorExpression>) -> EvaluatorExpression,
    ) -> Line<T> {
        self.verify_line(&val1);

        let mut eval = Evaluator::default();

        let input_expr1 = Box::new(eval.get_input(val1.inner).unwrap());
        eval.exprs.push(expr_fn(input_expr1));

        self.insert_component_with_output(eval)
    }

    fn evaluator_expr2<T: LineValue + ?Sized>(
        &self,
        val1: Line<impl LineValue + ?Sized>,
        val2: Line<impl LineValue + ?Sized>,
        expr_fn: impl FnOnce(Box<EvaluatorExpression>, Box<EvaluatorExpression>) -> EvaluatorExpression,
    ) -> Line<T> {
        self.verify_line(&val1);

        let mut eval = Evaluator::default();

        let input_expr1 = Box::new(eval.get_input(val1.inner).unwrap());
        let input_expr2 = Box::new(eval.get_input(val2.inner).unwrap());
        eval.exprs.push(expr_fn(input_expr1, input_expr2));

        self.insert_component_with_output(eval)
    }

    fn evaluator_expr3<T: LineValue + ?Sized>(
        &self,
        val1: Line<impl LineValue + ?Sized>,
        val2: Line<impl LineValue + ?Sized>,
        val3: Line<impl LineValue + ?Sized>,
        expr_fn: impl FnOnce(Box<EvaluatorExpression>, Box<EvaluatorExpression>, Box<EvaluatorExpression>) -> EvaluatorExpression,
    ) -> Line<T> {
        self.verify_line(&val1);

        let mut eval = Evaluator::default();

        let input_expr1 = Box::new(eval.get_input(val1.inner).unwrap());
        let input_expr2 = Box::new(eval.get_input(val2.inner).unwrap());
        let input_expr3 = Box::new(eval.get_input(val3.inner).unwrap());
        eval.exprs.push(expr_fn(input_expr1, input_expr2, input_expr3));

        self.insert_component_with_output(eval)
    }

    pub fn new_vector(&self, x: f64, y: f64, z: f64) -> Line<BVector3> {
        let expr = EvaluatorExpression::Vector(
            Box::new(EvaluatorExpression::Float(x)),
            Box::new(EvaluatorExpression::Float(y)),
            Box::new(EvaluatorExpression::Float(z)),
        );

        let mut eval = Evaluator::default();
        eval.exprs.push(expr);

        self.insert_component_with_output(eval)
    }

    make_bb_method!(sin, Sin, angle: BNumber, BNumber);
    make_bb_method!(cos, Cos, angle: BNumber, BNumber);
    make_bb_method!(tan, Tan, angle: BNumber, BNumber);
    make_bb_method!(sqrt, Sqrt, num: BNumber, BNumber);
    make_bb_method!(asin, Asin, num: BNumber, BNumber);
    make_bb_method!(acos, Acos, num: BNumber, BNumber);
    make_bb_method!(atan, Atan, num: BNumber, BNumber);
    make_bb_method!(atan2, Atan2, x: BNumber, y: BNumber, BNumber);
    make_bb_method!(exp, Exp, exponent: BNumber, BNumber);
    make_bb_method!(log, Log, num: BNumber, BNumber);
    make_bb_method!(pow, Pow, base: BNumber, exponent: BNumber, BNumber);
    make_bb_method!(abs, Abs, num: BNumber, BNumber);
    // compopnent wise absolute value
    make_bb_method!(absv, Abs, vec: BVector3, BVector3);
    make_bb_method!(sign, Sign, num: BNumber, BNumber);
    make_bb_method!(round, Round, num: BNumber, BNumber);
    make_bb_method!(floor, Floor, num: BNumber, BNumber);
    make_bb_method!(ceil, Ceil, num: BNumber, BNumber);
    make_bb_method!(max2, Max2, a: BNumber, b: BNumber, BNumber);
    make_bb_method!(max3, Max3, a: BNumber, b: BNumber, c: BNumber, BNumber);
    make_bb_method!(maxv, MaxV, vec: BVector3, BNumber);
    make_bb_method!(min2, Min2, a: BNumber, b: BNumber, BNumber);
    make_bb_method!(min3, Min3, a: BNumber, b: BNumber, c: BNumber, BNumber);
    make_bb_method!(minv, MinV, vec: BVector3, BNumber);

    pub fn b_if<T: LineValue + ?Sized>(&self, condition: Line<BNumber>, true_value: Line<T>, false_value: Line<T>) -> Line<T> {
        self.evaluator_expr3(condition, true_value, false_value, |a, b, c| {
            EvaluatorExpression::If {
                condition: a,
                true_value: b,
                false_value: c,
            }
        })
    }

    make_bb_method!(vector, Vector, x: BNumber, y: BNumber, z: BNumber, BVector3);
    make_bb_method_named!(new_rotation_between, MakeRotationBetween, from_vector: BVector3, to_vector: BVector3, BQuaternion);
    make_bb_method_named!(rotation_from_euler_angles, FromEuler, pitch: BNumber, yaw: BNumber, roll: BNumber, BQuaternion);
    make_bb_method!(rotation_from_euler_vector, FromEularV, vector: BVector3, BQuaternion);
    make_bb_method!(rotation_to_euler_vector, ToEularV, rotation: BQuaternion, BVector3);
    make_bb_method!(rotation_angle, Angle, rotation: BQuaternion, BNumber);
    make_bb_method!(rotation_axis, Axis, rotation: BQuaternion, BVector3);
    make_bb_method_named!(angle_between, AngleBetween, from_vector: BVector3, to_vector: BVector3, BNumber);
    make_bb_method_named!(set_x, SetX, vector: BVector3, x: BNumber, BVector3);
    make_bb_method_named!(set_y, SetY, vector: BVector3, y: BNumber, BVector3);
    make_bb_method_named!(set_z, SetZ, vector: BVector3, z: BNumber, BVector3);

    // properties
    make_bb_method!(x, GetX, vector: BVector3, BNumber);
    make_bb_method!(y, GetY, vector: BVector3, BNumber);
    make_bb_method!(z, GetZ, vector: BVector3, BNumber);

    make_bb_method!(magnitude, Magnitude, vector: BVector3, BNumber);
    make_bb_method!(square_magnitude, SquareMagnitude, vector: BVector3, BNumber);
    make_bb_method!(rotation_inverse, RotationInverse, rotation: BQuaternion, BQuaternion);

    // operators
    make_bb_method!(add, Add, a: BNumber, b: BNumber, BNumber);
    make_bb_method!(addv, Add, a: BVector3, b: BVector3, BVector3);
    make_bb_method!(concat, Add, a: BString, b: BString, BString);

    make_bb_method!(sub, Sub, a: BNumber, b: BNumber, BNumber);
    make_bb_method!(subv, Sub, a: BVector3, b: BVector3, BVector3);
    make_bb_method!(remove_instances, Sub, a: BString, b: BString, BString);

    make_bb_method!(cross, Cross, a: BVector3, b: BVector3, BVector3);

    make_bb_method!(mul, Mul, a: BNumber, b: BNumber, BNumber);
    make_bb_method!(dot, Mul, a: BVector3, b: BVector3, BNumber);
    make_bb_method!(rotate, Mul, a: BVector3, b: BQuaternion, BVector3);
    make_bb_method!(scale, Mul, a: BNumber, b: BVector3, BVector3);
    make_bb_method!(compose_rotations, Mul, a: BQuaternion, b: BQuaternion, BQuaternion);

    make_bb_method!(div, Div, a: BNumber, b: BNumber, BNumber);
    make_bb_method!(scaler_div, Div, a: BVector3, b: BNumber, BVector3);
    // divide with vector and quaternion ommited because
    // it makes more sense just to rotate by inverse of quaternion

    make_bb_method!(modulo, Mod, a: BNumber, b: BNumber, BNumber);

    make_bb_method!(eq, Eq, a: BNumber, b: BNumber, BNumber);
    make_bb_method!(ne, Ne, a: BNumber, b: BNumber, BNumber);
    make_bb_method!(gt, Gt, a: BNumber, b: BNumber, BNumber);
    make_bb_method!(gte, Gte, a: BNumber, b: BNumber, BNumber);
    make_bb_method!(lt, Lt, a: BNumber, b: BNumber, BNumber);
    make_bb_method!(lte, Lte, a: BNumber, b: BNumber, BNumber);
    
    make_bb_method!(not, Not, n: BNumber, BNumber);
    make_bb_method!(and, OpAnd, a: BNumber, b: BNumber, BNumber);
    make_bb_method!(or, OpOr, a: BNumber, b: BNumber, BNumber);
    make_bb_method!(false_coalesce, FalseCoalesce, a: BNumber, b: BNumber, BNumber);

    make_bb_method!(negate, Negate, n: BNumber, BNumber);
}

#[derive(Debug)]
pub enum EvaluatorExpression {
    InputA,
    InputB,
    InputC,
    InputD,
    InputE,
    Int(i64),
    Float(f64),
    Sin(Box<Self>),
    Cos(Box<Self>),
    Tan(Box<Self>),
    Sqrt(Box<Self>),
    Asin(Box<Self>),
    Acos(Box<Self>),
    Atan(Box<Self>),
    Atan2(Box<Self>, Box<Self>),
    Exp(Box<Self>),
    Log(Box<Self>),
    Pow(Box<Self>, Box<Self>),
    // absolute value also works component wise on a vector
    Abs(Box<Self>),
    Sign(Box<Self>),
    Round(Box<Self>),
    Floor(Box<Self>),
    Ceil(Box<Self>),
    Max2(Box<Self>, Box<Self>),
    Max3(Box<Self>, Box<Self>, Box<Self>),
    // returns maximum component of the vector
    MaxV(Box<Self>),
    Min2(Box<Self>, Box<Self>),
    Min3(Box<Self>, Box<Self>, Box<Self>),
    // returns minimum component of the vector
    MinV(Box<Self>),
    If {
        condition: Box<Self>,
        true_value: Box<Self>,
        false_value: Box<Self>,
    },
    Vector(Box<Self>, Box<Self>, Box<Self>),
    MakeRotationBetween {
        from_vector: Box<Self>,
        to_vector: Box<Self>,
    },
    FromEuler {
        pitch: Box<Self>,
        yaw: Box<Self>,
        roll: Box<Self>,
    },
    // crates rotation with x being x rotation, y rotation, and z rotation
    FromEularV(Box<Self>),
    // converts rotation back into vector of angles
    ToEularV(Box<Self>),
    // returns angle of rotation (based on real part of quaternion?)
    Angle(Box<Self>),
    // returns the rotation axis of quaternion
    Axis(Box<Self>),
    AngleBetween {
        from_vector: Box<Self>,
        to_vector: Box<Self>,
    },
    SetX {
        vector: Box<Self>,
        x: Box<Self>,
    },
    SetY {
        vector: Box<Self>,
        y: Box<Self>,
    },
    SetZ {
        vector: Box<Self>,
        z: Box<Self>,
    },
    // output vector of previous frame for the given index into the outputs
    OutputV(Box<Self>),
    // output number of previous frame for the given index into the outputs
    Output(Box<Self>),
    GetX(Box<Self>),
    GetY(Box<Self>),
    GetZ(Box<Self>),
    Magnitude(Box<Self>),
    SquareMagnitude(Box<Self>),
    RotationInverse(Box<Self>),
    // works for 2 numbers, string, or vectors
    Add(Box<Self>, Box<Self>),
    // works for 2 numbers, strings (lhs with instances of rhs removed), or vectors
    Sub(Box<Self>,Box<Self>),
    // vector cross product
    Cross(Box<Self>, Box<Self>),
    // works for 2 numbers, 2 vectors (dot product), rotation then vector (rotate vector),
    // number then vector, vector then number, 2 rotations (new rotation where lhs rotate after rhs)
    Mul(Box<Self>, Box<Self>),
    // works for 2 numbers, vector / number, vector / rotation (apply inverse rotation)
    Div(Box<Self>, Box<Self>),
    // workse for 2 numbers (remainder)
    Mod(Box<Self>, Box<Self>),
    Eq(Box<Self>, Box<Self>),
    Ne(Box<Self>, Box<Self>),
    Gt(Box<Self>, Box<Self>),
    Gte(Box<Self>, Box<Self>),
    Lt(Box<Self>, Box<Self>),
    Lte(Box<Self>, Box<Self>),
    Not(Box<Self>),
    OpAnd(Box<Self>, Box<Self>),
    OpOr(Box<Self>, Box<Self>),
    FalseCoalesce(Box<Self>, Box<Self>),
    Negate(Box<Self>),
}

impl EvaluatorExpression {
    fn from_input_index(n: usize) -> Option<Self> {
        match n {
            0 => Some(Self::InputA),
            1 => Some(Self::InputB),
            2 => Some(Self::InputC),
            3 => Some(Self::InputD),
            4 => Some(Self::InputE),
            _ => None,
        }
    }
}

impl Display for EvaluatorExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputA => write!(f, "a"),
            Self::InputB => write!(f, "b"),
            Self::InputC => write!(f, "c"),
            Self::InputD => write!(f, "d"),
            Self::InputE => write!(f, "e"),
            Self::Int(val) => write!(f, "{val}"),
            Self::Float(val) => write!(f, "{val}"),
            Self::Sin(val) => write!(f, "Sin({val})"),
            Self::Cos(val) => write!(f, "Cos({val})"),
            Self::Tan(val) => write!(f, "Tan({val})"),
            Self::Sqrt(val) => write!(f, "Sqrt({val})"),
            Self::Asin(val) => write!(f, "Asin({val})"),
            Self::Acos(val) => write!(f, "Acos({val})"),
            Self::Atan(val) => write!(f, "Atan({val})"),
            Self::Atan2(val1, val2) => write!(f, "Atan({val1}, {val2})"),
            Self::Exp(val) => write!(f, "Exp({val})"),
            Self::Log(val) => write!(f, "Log({val})"),
            Self::Pow(val1, val2) => write!(f, "Pow({val1}, {val2})"),
            Self::Abs(val) => write!(f, "Abs({val})"),
            Self::Sign(val) => write!(f, "Sign({val})"),
            Self::Round(val) => write!(f, "Round({val})"),
            Self::Floor(val) => write!(f, "Floor({val})"),
            Self::Ceil(val) => write!(f, "Ceil({val})"),
            Self::Max2(val1, val2) => write!(f, "Max({val1}, {val2})"),
            Self::Max3(val1, val2, val3) => write!(f, "Max({val1}, {val2}, {val3})"),
            Self::MaxV(val) => write!(f, "Max({val})"),
            Self::Min2(val1, val2) => write!(f, "Min({val1}, {val2})"),
            Self::Min3(val1, val2, val3) => write!(f, "Min({val1}, {val2}, {val3})"),
            Self::MinV(val) => write!(f, "Min({val})"),
            Self::If {
                condition,
                true_value,
                false_value,
            } => write!(f, "If({condition}, {true_value}, {false_value})"),
            Self::Vector(val1, val2, val3) => write!(f, "Vector({val1}, {val2}, {val3})"),
            Self::MakeRotationBetween {
                from_vector,
                to_vector,
            } => write!(f, "FromToRot({from_vector}, {to_vector}"),
            Self::FromEuler {
                pitch,
                yaw,
                roll,
            } => write!(f, "FromEuler({pitch}, {yaw}, {roll})"),
            Self::FromEularV(val) => write!(f, "FromEuler({val})"),
            Self::ToEularV(val) => write!(f, "ToEuler({val})"),
            Self::Angle(val) => write!(f, "Angle({val})"),
            Self::Axis(val) => write!(f, "Axis({val})"),
            Self::AngleBetween {
                from_vector,
                to_vector,
            } => write!(f, "Angle({from_vector}, {to_vector})"),
            Self::SetX {
                vector,
                x,
            } => write!(f, "setX({vector}, {x})"),
            Self::SetY {
                vector,
                y,
            } => write!(f, "setX({vector}, {y})"),
            Self::SetZ {
                vector,
                z,
            } => write!(f, "setX({vector}, {z})"),
            Self::OutputV(val) => write!(f, "outputV({val})"),
            Self::Output(val) => write!(f, "output({val})"),
            Self::GetX(val) => write!(f, "({val}).x"),
            Self::GetY(val) => write!(f, "({val}).y"),
            Self::GetZ(val) => write!(f, "({val}).z"),
            Self::Magnitude(val) => write!(f, "({val}).magnitude"),
            Self::SquareMagnitude(val) => write!(f, "({val}).sqrMagnitude"),
            Self::RotationInverse(val) => write!(f, "({val}).inverse"),
            Self::Add(lhs, rhs) => write!(f, "({lhs}) + ({rhs})"),
            Self::Sub(lhs, rhs) => write!(f, "({lhs}) - ({rhs})"),
            Self::Cross(lhs, rhs) => write!(f, "({lhs}) x ({rhs})"),
            Self::Mul(lhs, rhs) => write!(f, "({lhs}) * ({rhs})"),
            Self::Div(lhs, rhs) => write!(f, "({lhs}) / ({rhs})"),
            Self::Mod(lhs, rhs) => write!(f, "({lhs}) % ({rhs})"),
            Self::Eq(lhs, rhs) => write!(f, "({lhs}) = ({rhs})"),
            Self::Ne(lhs, rhs) => write!(f, "({lhs}) != ({rhs})"),
            Self::Gt(lhs, rhs) => write!(f, "({lhs}) > ({rhs})"),
            Self::Gte(lhs, rhs) => write!(f, "({lhs}) >= ({rhs})"),
            Self::Lt(lhs, rhs) => write!(f, "({lhs}) < ({rhs})"),
            Self::Lte(lhs, rhs) => write!(f, "({lhs}) <= ({rhs})"),
            Self::Not(val) => write!(f, "!({val})"),
            Self::OpAnd(lhs, rhs) => write!(f, "({lhs}) & ({rhs})"),
            Self::OpOr(lhs, rhs) => write!(f, "({lhs}) | ({rhs})"),
            Self::FalseCoalesce(lhs, rhs) => write!(f, "({lhs}) or ({rhs})"),
            Self::Negate(val) => write!(f, "-({val})"),
        }
    }
}