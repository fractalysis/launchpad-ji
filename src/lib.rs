use nih_plug::prelude::*;
use std::sync::Arc;

struct LaunchpadJI {
    params: Arc<LaunchpadJIParams>
}

#[derive(Default, Params)]
struct LaunchpadJIParams {}

impl Default for LaunchpadJI {
    fn default() -> Self {
        Self {
            params: Arc::new(LaunchpadJIParams::default()),
        }
    }
}

impl Plugin for LaunchpadJI {
    const NAME: &'static str = "Launchpad JI";
    const VENDOR: &'static str = "Fractalysis";
    const URL: &'static str = "";
    const EMAIL: &'static str = "fractalysisofficial@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // This plugin doesn't have any audio IO
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn {
                    timing,
                    voice_id,
                    channel,
                    note,
                    velocity,
                } => {

                }

                _ => ()

            }
        }

        ProcessStatus::Normal
    }
}

impl Vst3Plugin for LaunchpadJI {
    const VST3_CLASS_ID: [u8; 16] = *b"LaunchpadMPEJIfx";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Tools];
}


nih_export_vst3!(LaunchpadJI);