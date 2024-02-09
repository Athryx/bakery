mod evaluator;
mod line_value;

use std::cell::RefCell;
use std::{marker::PhantomData, path::Path};
use std::io;

use uuid::{Uuid, uuid};

pub use line_value::*;
use crate::ftd_data::{ftd_uuid_to_uuid, BlockData, BlueprintData, DataEntry, SectionData, SectionId, Vector2};
use evaluator::Evaluator;

/// Ids used for components to avoid interfering with other ids
const COMPONENT_ID_START: u32 = 72542;

struct BreadboardWireMap {
    /// 2d array where first index is component id, 2nd is output index
    data: Vec<Vec<Uuid>>,
}

impl BreadboardWireMap {
    fn new(num_components: usize) -> Self {
        let mut data = Vec::with_capacity(num_components);
        for _ in 0..num_components {
            data.push(Vec::new());
        }

        BreadboardWireMap {
            data,
        }
    }

    fn set_num_outputs(&mut self, component_index: usize, num_outputs: usize) {
        for _ in 0..num_outputs {
            self.data[component_index].push(Uuid::new_v4());
        }
    }

    fn get_output_uuid(&self, line: LineInner) -> &Uuid {
        &self.data[line.component_index][line.output_index]
    }

    fn get_component_data_section_with_inputs_and_outputs(&mut self, component_index: usize, component: &dyn Component) -> SectionData {
        self.set_num_outputs(component_index, component.num_outputs());

        let mut input_bytes = Vec::new();
        for line in component.inputs() {
            // I don't really know what this uuid is for but ftd needs 2 uuids for input lines
            input_bytes.extend_from_slice(Uuid::new_v4().as_bytes());

            input_bytes.extend_from_slice(self.get_output_uuid(*line).as_bytes());
        }

        let mut output_bytes = Vec::new();
        for output_uuid in self.data[component_index].iter() {
            output_bytes.extend_from_slice(output_uuid.as_bytes());
        }

        component.section_data()
            .with_entry(900, DataEntry::Bytes(input_bytes))
            .with_entry(901, DataEntry::Bytes(output_bytes))
    }
}

#[derive(Default)]
pub struct Breadboard {
    components: RefCell<Vec<Box<dyn Component>>>,
}

impl Breadboard {
    fn block_data(&self) -> BlockData {
        let components = self.components.borrow();

        let mut wire_map = BreadboardWireMap::new(components.len());
        let mut data = BlockData::default();

        // don't know what these sections do, but ftd seems to generate these empty sections for a breadboard
        data.add_section_data(3000.into(), SectionData::default());
        data.add_section_data(72541.into(), SectionData::default());

        let mut breadboard_main_section = SectionData::default();
        for (i, component) in components.iter().enumerate() {
            breadboard_main_section.add_entry(
                (2 * i).try_into().unwrap(),
                DataEntry::Uuid(component.uuid()),
            );

            let component_id = COMPONENT_ID_START + u32::try_from(i).unwrap();

            breadboard_main_section.add_entry(
                (2 * i + 1).try_into().unwrap(),
                DataEntry::U32(component_id),
            );

            let mut component_section_data = wire_map.get_component_data_section_with_inputs_and_outputs(i, &**component);

            let position = ComponentPosition::new(200.0 * i as f32, 0.0);
            position.set_section_data_position(&mut component_section_data);

            data.add_section_data(SectionId::new(component_id), component_section_data);
        }

        // breadboard looks at this section to instantiate components
        data.add_section_data(9999.into(), breadboard_main_section);

        data
    }

    pub fn save_to_prefab_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut data = BlueprintData::default();
        data.add_block_data(0.into(), self.block_data());
        let data = data.serialize_to_bp_data_string();

        let mut bp_file = String::from(r#"{"FileModelVersion":{"Major":1,"Minor":0},"Name":"TEST_BREADBOARD","Version":0,"SavedTotalBlockCount":1,"SavedMaterialCost":10.0,"ContainedMaterialCost":0.0,"ItemDictionary":{"227":"5ef97d26-1196-4b1a-ba1d-fd539c26b684","0":"75a78e48-0848-45ee-9df2-e2b328c1933d"},"Blueprint":{"ContainedMaterialCost":0.0,"CSI":[-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0,-1.0],"COL":null,"SCs":[],"BLP":["0,0,0"],"BLR":[0],"BP1":null,"BP2":null,"BCI":[0],"BEI":null,"BlockData":""#);
        bp_file.push_str(&data);
        bp_file.push_str(r#"","VehicleData":"sct0AAAAAAAA","designChanged":false,"blueprintVersion":0,"blueprintName":"TEST_BREADBOARD","SerialisedInfo":{"JsonDictionary":{},"IsEmpty":true},"Name":null,"ItemNumber":0,"LocalPosition":"0,0,0","LocalRotation":"0,0,0,0","ForceId":0,"TotalBlockCount":1,"MaxCords":"1,1,1","MinCords":"0,0,0","BlockIds":[227],"BlockState":null,"AliveCount":1,"BlockStringData":null,"BlockStringDataIds":null,"GameVersion":"3.8.0.4","PersistentSubObjectIndex":-1,"PersistentBlockIndex":-1,"AuthorDetails":{"Valid":true,"ForeignBlocks":0,"CreatorId":"0ab41fc3-fd53-4843-becf-7608b7c315b7","ObjectId":"5bb43b25-8e79-4e92-9db3-076b363114a7","CreatorReadableName":"DeltaForce","HashV1":"6831413c85b3e408740dc00f5580382c"},"BlockCount":1}}"#);
        
        std::fs::write(path, bp_file)
    }

    fn verify_line<T: LineValue>(&self, line: Line<T>) {
        assert!(self as *const _ as usize == line.breadboard as *const _ as usize, "invalid line passed into breadboard")
    }

    /// Inserts the component into the breadboard and returns its index
    fn insert_component<C: Component + 'static>(&self, component: C) -> usize {
        let mut components = self.components.borrow_mut();
        components.push(Box::new(component));
        components.len() - 1
    }

    /// Inserts a component with 1 output
    fn insert_component_with_output<C: Component + 'static, T: LineValue>(&self, component: C) -> Line<T> {
        Line::new(self, self.insert_component(component), 0)
    }

    pub fn constant(&self, n: f32) -> Line<BNumber> {
        self.insert_component_with_output(Constant {
            n: n.clamp(-10000.0, 10000.0),
        })
    }

    pub fn random_number(&self, min: f32, max: f32) -> Line<BNumber> {
        let min = min.clamp(-10000.0, 10000.0);
        let max = max.clamp(min, 10000.0);

        self.insert_component_with_output(RandomInput {
            min,
            max,
        })
    }

    pub fn altitude(&self, altitude_type: AltitudeOutputType) -> Line<BNumber> {
        self.insert_component_with_output(Altitude {
            typ: altitude_type,
        })
    }

    pub fn position(&self) -> Line<BVector3> {
        self.insert_component_with_output(Position)
    }

    pub fn speed(&self, speed_type: SpeedOutputType) -> Line<BNumber> {
        self.insert_component_with_output(Speed {
            typ: speed_type,
        })
    }
 
    pub fn velocity(&self, speed_type: VelocityOutputType) -> Line<BVector3> {
        self.insert_component_with_output(Velocity {
            typ: speed_type,
        })
    }
}

/// This contains all info returnd by the primary target info component
#[derive(Clone, Copy)]
pub struct TargetInfoOutputs<'a> {
    pub present: Line<'a, BNumber>,
    pub distance: Line<'a, BNumber>,
    pub altitude: Line<'a, BNumber>,
    /// Target bearing relative to our forward in degress (range of [-180, 180])
    pub bearing: Line<'a, BNumber>,
    pub position: Line<'a, BVector3>,
    pub velocity: Line<'a, BVector3>,
    pub volume: Line<'a, BNumber>,
}

impl Breadboard {
    pub fn target_info(&self) -> TargetInfoOutputs {
        let component_id = self.insert_component(TargetInfo);

        TargetInfoOutputs {
            present: Line::new(self, component_id, 0),
            distance: Line::new(self, component_id, 1),
            altitude: Line::new(self, component_id, 2),
            bearing: Line::new(self, component_id, 3),
            position: Line::new(self, component_id, 4),
            velocity: Line::new(self, component_id, 5),
            volume: Line::new(self, component_id, 6),
        }
    }

    // TODO: maybe allow vectors, I think multiply tachnically allows it in some cases
    pub fn multiply<'a, T: InputGroup<BNumber>>(&'a self, inputs: &T, multiplier: f32) -> Line<'a, BNumber> {
        let multiplier = multiplier.clamp(-100.0, 100.0);

        self.insert_component_with_output(Multiply {
            multiplier,
            inputs: inputs.as_vec(),
        })
    }

    // TODO: maybe allow vectore here as well, switch also works with vectors, but the behavior is very wierd (vector magnitude is passed through)
    pub fn switch<'a>(&'a self, passthrough: Line<'a, BNumber>, switch_signal: Line<'a, BNumber>, options: SwitchOptions) -> Line<'a, BNumber> {
        self.verify_line(passthrough);
        self.verify_line(switch_signal);

        self.insert_component_with_output(Switch {
            inputs: [passthrough.inner, switch_signal.inner],
            threshhold: options.threshhold.clamp(-10000.0, 10000.0),
            open_value: options.open_value.clamp(-10000.0, 10000.0),
        })
    }
}

struct ComponentPosition {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl ComponentPosition {
    pub fn new(x: f32, y: f32) -> ComponentPosition {
        ComponentPosition {
            x,
            y,
            width: 25.0,
            height: 25.0,
        }
    }
}

impl ComponentPosition {
    fn set_section_data_position(&self, section_data: &mut SectionData) {
        section_data.add_entry(8000, DataEntry::F32(self.x));
        section_data.add_entry(8001, DataEntry::F32(self.y));
        section_data.add_entry(8002, DataEntry::F32(self.width));
        section_data.add_entry(8003, DataEntry::F32(self.height));
    }
}

trait Component {
    fn ftd_uuid(&self) -> Uuid;

    fn uuid(&self) -> Uuid {
        ftd_uuid_to_uuid(self.ftd_uuid())
    }

    fn section_data(&self) -> SectionData;
    fn num_outputs(&self) -> usize;
    fn inputs(&self) -> &[LineInner];
}

/// Represents the output line of a certain breadboard component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineInner {
    component_index: usize,
    output_index: usize,
}

pub struct Line<'a, T: LineValue + ?Sized> {
    inner: LineInner,
    // the line pretends it owns a type T inside the breadboard in its wires
    breadboard: &'a Breadboard,
    _marker: PhantomData<T>,
}

impl<'a, T: LineValue> Line<'a, T> {
    fn new(breadboard: &'a Breadboard, component_index: usize, output_index: usize) -> Self {
        Line {
            inner: LineInner {
                component_index,
                output_index,
            },
            breadboard,
            _marker: PhantomData,
        }
    }
}

impl<T: LineValue + ?Sized> Clone for Line<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: LineValue + ?Sized> Copy for Line<'_, T> {}

// TODO: support constant strings
#[derive(Debug)]
struct Constant {
    n: f32,
}

impl Component for Constant {
    fn ftd_uuid(&self) -> Uuid {
        uuid!("9142c70d-7833-41cd-804d-554e990b6904")
    }

    fn section_data(&self) -> SectionData {
        SectionData::default()
            .with_entry(0, DataEntry::F32(self.n))
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn inputs(&self) -> &[LineInner] {
        &[]
    }
}

#[derive(Debug)]
struct RandomInput {
    min: f32,
    max: f32,
}

impl Component for RandomInput {
    fn ftd_uuid(&self) -> Uuid {
        uuid!("268b7db2-bccf-41fd-8cfa-3f21d2f2bacb")
    }

    fn section_data(&self) -> SectionData {
        SectionData::default()
            .with_entry(0, DataEntry::Vector2(Vector2::new(self.min, self.max)))
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn inputs(&self) -> &[LineInner] {
        &[]
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AltitudeOutputType {
    SeaLevel,
    WaveLevel,
    TerrainLevel,
    TerrainAndWave,
    TerrainAndSea,
}

#[derive(Debug)]
struct Altitude {
    typ: AltitudeOutputType,
}

impl Component for Altitude {
    fn ftd_uuid(&self) -> Uuid {
        uuid!("ae46572b-dff8-4153-97dc-146108f3a64f")
    }

    fn section_data(&self) -> SectionData {
        SectionData::default()
            .with_entry(0, DataEntry::U32(self.typ as u32))
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn inputs(&self) -> &[LineInner] {
        &[]
    }
}

#[derive(Debug)]
struct Position;

impl Component for Position {
    fn ftd_uuid(&self) -> Uuid {
        uuid!("e20d6a3a-c0b9-4665-8749-7a85c40afabe")
    }

    fn section_data(&self) -> SectionData {
        SectionData::default()
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn inputs(&self) -> &[LineInner] {
        &[]
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedOutputType {
    Magnitude = 0,
    ForwardsMagnitude = 3,
}

#[derive(Debug)]
struct Speed {
    typ: SpeedOutputType,
}

impl Component for Speed {
    fn ftd_uuid(&self) -> Uuid {
        uuid!("c8f64443-b81f-4b75-8105-18cd6e453539")
    }

    fn section_data(&self) -> SectionData {
        SectionData::default()
            .with_entry(0, DataEntry::U32(self.typ as u32))
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn inputs(&self) -> &[LineInner] {
        &[]
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VelocityOutputType {
    Magnitude = 0,
    ForwardsMagnitude = 3,
}

// velocity is seperated from speed even though they are the same underlying component
// because these are the speed modes outputing a vector
#[derive(Debug)]
struct Velocity {
    typ: VelocityOutputType,
}

impl Component for Velocity {
    fn ftd_uuid(&self) -> Uuid {
        uuid!("c8f64443-b81f-4b75-8105-18cd6e453539")
    }

    fn section_data(&self) -> SectionData {
        SectionData::default()
            .with_entry(0, DataEntry::U32(self.typ as u32))
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn inputs(&self) -> &[LineInner] {
        &[]
    }
}

#[derive(Debug)]
struct TargetInfo;

impl Component for TargetInfo {
    fn ftd_uuid(&self) -> Uuid {
        uuid!("5390bcf0-d09d-40b8-99a3-8d3752e656c6")
    }

    fn section_data(&self) -> SectionData {
        SectionData::default()
    }

    fn num_outputs(&self) -> usize {
        7
    }

    fn inputs(&self) -> &[LineInner] {
        &[]
    }
}

#[derive(Debug)]
struct Multiply {
    multiplier: f32,
    inputs: Vec<LineInner>,
}

impl Component for Multiply {
    fn ftd_uuid(&self) -> Uuid {
        uuid!("930e5331-cecf-408a-8d90-dac6b479d5b0")
    }

    fn section_data(&self) -> SectionData {
        SectionData::default()
            .with_entry(0, DataEntry::F32(self.multiplier))
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn inputs(&self) -> &[LineInner] {
        self.inputs.as_slice()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SwitchOptions {
    threshhold: f32,
    open_value: f32,
}

impl Default for SwitchOptions {
    fn default() -> Self {
        SwitchOptions {
            threshhold: 0.5,
            open_value: 0.0,
        }
    }
}

#[derive(Debug)]
struct Switch {
    // first input is passthrough, second is switch signal
    inputs: [LineInner; 2],
    threshhold: f32,
    open_value: f32,
}

impl Component for Switch {
    fn ftd_uuid(&self) -> Uuid {
        uuid!("581de01e-3754-45f6-9133-f51443844eca")
    }

    fn section_data(&self) -> SectionData {
        SectionData::default()
            .with_entry(0, DataEntry::F32(self.threshhold))
            .with_entry(1, DataEntry::F32(self.open_value))
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn inputs(&self) -> &[LineInner] {
        &self.inputs
    }
}