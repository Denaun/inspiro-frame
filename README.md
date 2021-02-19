# InspiroFrame

## An [InspiroBot]-based picture frame for [RaspberryPI].

[InspiroBot]: https://inspirobot.me
[RaspberryPi]: https://www.raspberrypi.org

## Example configuration

```toml
[gpio]
5 = ["inspiro-frame", "next"]
6 = ["uhubctl", "-l", "1-1", "-a", "3"]
```