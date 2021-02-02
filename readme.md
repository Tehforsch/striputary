# Striputary
Striputary is a program to record an audio stream from spotify (but could be easily extended to work for other streaming services) and convert the recorded audio into individual songs. 

Striputary relies on 
* Pulseaudio and parec for recording audio
* ffmpeg for cutting the audio buffer into songs, adding metadata and converting to the different audio formats
* D-Bus (via the [dbus-rs crate](https://github.com/diwic/dbus-rs)) to read song information (such as artist, album, title and song length) that is sent from the media player and to control playback

The problem striputary tries to solve is cutting the stream into individual songs. Getting this exactly right is somewhat tricky. Striputary records D-bus information while recording and will therefore know exactly which songs were recorded in which order. However, the D-bus signal does not come at the exact millisecond a song begins. For song transitions with very little silence, this is unacceptable. Fortunately, the signal includes the exact song length. That means that if we knew exactly where a single song begins in the audio stream, we know where to cut all others as well. Therefore, the problem comes down to finding the offset for all the cuts.

Currently there are two ways to find this offset:

* Automatic mode: In this mode, striputary effectively calculates the volume averaged over all cut positions and chooses the offset such that it minimizes this average volume. This is based on the assumption that most song transition contain some silence. It works best when a lot of songs were recorded because that constrains the cut position better. I recommend trying this mode and to try manual mode if the result is unsatisfactory.
* Manual mode: Striputary will ask the user for an offset and then use this for cutting the songs.

## Installation
```bash
git clone https://github.com/tehforsch/striputary
cargo build --release
```

## Usage
### Recording a stream
Begin by opening spotify and start the first song of a playlist you want to record.
Then run
```
striputary outputDirectory record
```

Striputary should now begin by creating a new pulseaudio sink and redirecting the spotify output to that sink. This means you should not hear any audio from spotify anymore. (You should be able to listen to audio on your computer normally while striputary is recording, as long as you do not play back to the recording sink).
Striputary will now begin recording and after a few seconds, you should see the song being rewinded to the beginning and playback should begin shortly after. (Don't worry, this happens only once and it is there to ensure we fully record the first song with some space around it in the audio buffer).

Once the playlist is finished, striputary will realize that playback has stopped and stop recording. You can also interrupt the recording manually by either stopping the playback in spotify or pressing Ctrl+C in striputary. Any songs that were not recorded fully will be ignored from here on.

### Cutting into songs
Run in automatic mode:
```
striputary outputDirectory cut auto
```
or, if you wanted to use manual mode with an offset of -0.5:
```
striputary outputDirectory cut manual -0.5
```

When all goes well, striputary should create a music folder in outputDirectory that should contain your music in .flac format. I recommend listening to a few songs to see if the recording is OK. 

### Meta-data
The meta-data added is very rudimentary. The only meta-data the resulting files contain will be the 
* Title
* Album
* First Artist
* Track number
in spotify.
Since this is unsatisfactory for most people, I recommend using [beets](http://beets.io/) to add meta data your music. So far, every album recorded with striputary has been recognized by beets immediately upon running
```
beet import outputDirectory/music
```

## Notes
### Other services
Implementing this for other services comes down to simply adding the corresponding dbus bus name and the pulse audio sink name entry in src/service_config.rs.
