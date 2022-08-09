# Striputary
Striputary is a program to record an audio stream from spotify (but could be easily extended to work for other streaming services) and convert the recorded audio into individual songs. 
![Graphical user interface](https://github.com/tehforsch/striputary/blob/master/pics/gui.png?raw=true)


Striputary relies on 
* Pulseaudio and parec for recording audio
* [ffmpeg](https://github.com/FFmpeg/FFmpeg/tree/master) for cutting the audio buffer into songs, adding metadata and converting to the different audio formats
* D-Bus (via the [dbus-rs crate](https://github.com/diwic/dbus-rs)) to read song information (such as artist, album, title and song length) that is sent from the media player and to control playback
plus optional dependencies:
* [egui](https://github.com/emilk/egui) for the gui.
* [rodio](https://github.com/RustAudio/rodio) for the audio playback.

## Installation
```bash
git clone https://github.com/tehforsch/striputary
cargo install --path .
```

## Usage
If the installation was succesful, it should be possible to start striputary via `striputary PATH_TO_OUTPUT_DIRECTORY` where `PATH_TO_OUTPUT_DIRECTORY` is the path into which the recorded files will be saved.
If you do not want to specify the output directory every time, you can configure it by adding the line

```
output_dir: PATH_TO_OUTPUT_DIRECTORY
```

in `XDG_CONFIG_HOME/striputary/config.yaml` (most likely `~/.config/striputary/config.yaml`)

### Recording a stream
Begin by opening spotify and starting the first song of a playlist you want to record and press the "record new session" button in striputary.

Striputary should now begin by creating a new pulseaudio sink and redirecting the spotify output to that sink. This means you should not hear any audio from spotify anymore. (You can still listen to audio on your computer normally while striputary is recording without ruining the recording, as long as you do not play back to the recording sink.)
Striputary will now begin recording and after a few seconds, you should see the song being rewinded to the beginning and playback should begin shortly after. (Don't worry, this happens only once and it is there to ensure we fully record the first song in the audio buffer).

Once the playlist is finished, striputary will realize that playback has stopped and stop recording. You can also interrupt the recording manually by stopping the playback in spotify. Any songs that were not recorded fully will be ignored from here on.

### Cutting into songs
So far, Striputary has only recorded the music into a large buffer, but we want to cut music into pieces ~~this is my last resort~~. To do select the recorded session in striputary (if you just finished recording, this should be the selected session).

Striputary automatically guesses the correct cut positions but this is hard to do in general (see [Details](#details) ). In the GUI, you will see the waveform around each of the cut positions. If you're unhappy with the cut position at any point, you can adjust the position by clicking on the waveform. In order to hear how the beginning of the last clicked song would sound like, press Space and the first few seconds of the song should be played back.
To scroll down/up use the arrow keys. Once you are happy with the position of the cut marker, press the "Cut" button. Cutting will take some time (a few seconds per song, typically).

Once finished, the cut songs are contained in the `music` subfolder of the output directory. The songs are available in `.opus` format.

### Meta-data
The meta-data added is very rudimentary. The only meta-data the resulting files contain will be the 
* Title
* Album
* First Artist
* Track number

Since this is unsatisfactory for most people, I recommend using [beets](http://beets.io/) to add meta data your music. So far, every album recorded with striputary has been recognized by beets immediately upon running
```
beet import outputDirectory/music
```

## Details
The problem striputary tries to solve is cutting the stream into individual songs. Getting this exactly right is somewhat tricky. Striputary records D-bus information while recording and will therefore know exactly which songs were recorded in which order. However, the D-bus signal does not come at the exact millisecond a song begins. For song transitions with very little silence, this is unacceptable. Fortunately, the signal includes the exact song length. That means that if we knew exactly where a single song begins in the audio stream, we know where to cut all others as well. Therefore, the problem comes down to finding the offset for all the cuts.

In order to provide a decent guess for the cut position, striputary effectively calculates the volume averaged over all cut positions and chooses the offset such that it minimizes this average volume. This is based on the assumption that most song transition contain some silence. 

![Average volume at cuts over cut offset](https://github.com/tehforsch/striputary/blob/master/pics/volumePlot.png?raw=true)

Automatic offset detection works best when a number of songs were recorded because that constrains the cut position better. I find that it works almost flawlessly when recording an entire album, for example. Once the recording becomes a lot longer (hundreds of songs), the offsets tend to shift very slightly over time for some reason I haven't been able to understand yet. 

## Notes
### Other services
Currently, this is only implemented for spotify. The approach doesn't rely on spotify-specific details though, so implementing this for other streaming services should be straightforward (It comes down to simply adding the corresponding dbus bus name and the pulse audio sink name entry in `src/service_config.rs`). 

## Disclaimer
Disclaimer: In an ideal world, you could use such recordings to reduce your personal dependence on large companies and simulatenously save some money which could then be used to support the actual artists whose music you are listening to (who get virtually nothing from their music being streamed). However, recording music off of streaming services is not only against the terms of service of pretty much all streaming providers but also possibly illegal and obviously immoral. Don't do it! This software has not been written with the idea of it being used but purely for educational purposes (handling audio files is lots of fun!).
