# bevy_trebuchet
### Bevy crate for creation  of an ancient siege weapon known as a trebuchet.
Avian3D is used as a physics engine. (Pure physics, no animation used)

## Why ?
I decided to make a trebuchet from the [Project Siege](https://github.com/xenon615/siege) as a crate.  
Why ? May be for convenience, may be for learning how to make crates, IDK :) .  

![Trebuchet](img/image1.png)
## Video 
[![Watch the video](https://img.youtube.com/vi/K_OJGsxl0fk/maxresdefault.jpg)](https://youtu.be/K_OJGsxl0fk)

## Disclaimer
Actually its my first crate made for Bevy or Rust. So  maybe I didn't do it the way it should be or the way it's usually done.
Keep this in mind if you plan to use it. However, you free to copy, clone and correct it at your own discretion.
Also, I would be glad to receive any advice. ([xenon615 on Discord](https://discordapp/users/xenon615))

## Usage
Add to your dependencies
```
avian3d = "0.3.1"
bevy_trebuchet = {git = "https://github.com/xenon615/bevy_trebuchet"}
```

```rust
use bevy_trebuchet::{NewTrebuchets, TrebuchetPlugin};

```

```rust

.add_plugins(TrebuchetPlugin::default())

```
In addition to the default settings, there are the following settings:
```rust

pub struct TrebuchetPlugin {
    /// the density of the counterweight material, well you get the idea, the heavier it is, the further the projectile will be thrown, but it can fall apart, 9.5 by default
    pub counterweight_density: f32,
    /// layer mask of ball collider, 666 by default
    pub ball_layer_mask: u32, 
    /// dot product between direction from trebuchet center to ball  and Y axis. Working values are approximately between 0.7 and 0.99.lower value means steeper trajectory
    pub unhooking_dot: f32,
    /// again, nothing incomprehensible, the denser the harder it will hit, if it can take off, of course)
    pub ball_density: f32
}
```

In order for the trebuchet(s) to appear, you need to do the following:

```rust
    cmd.trigger(NewTrebuchets(vec![
        Transform::from_xyz(0., 0., 0.),
        Transform::from_xyz(15., 0., 0.),
    ]));

```



Please refer to /examples/basic.rs for example. 

Also you can run for demo.

```
cargo run  --example basic

```

## To-Do
All we can do now is place the trebuchet in a certain position.
Therefore, the immediate plans are to allow it to be installed at any angle relative to the Y axis
(currently, an attempt to install it not strictly north causes its destruction) (IDK at the moment, why?)  
Next, at the moment trebuchet starts working automatically after creation  after which it starts working automatically.
So need add some control ( start / stop at least)  
Well, I'll add something else.



