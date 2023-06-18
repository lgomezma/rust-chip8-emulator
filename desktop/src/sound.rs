use rodio::OutputStreamHandle;
use std::io::BufReader;
use std::thread;
use std::time::Duration;

pub fn make_sound(beep: bool, stream_handle: &OutputStreamHandle) {
    if !beep {
        return;
    }
    let file = std::fs::File::open("assets/assets_beep.wav").unwrap();
    let beep_sound = stream_handle.play_once(BufReader::new(file)).unwrap();

    beep_sound.set_volume(0.2);
    thread::sleep(Duration::from_millis(100));
    beep_sound.stop();
    beep_sound.detach();
}