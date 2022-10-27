from pathlib import Path
from json import loads
import subprocess


def main(folder: Path):
    concatpath = folder.joinpath("concatfile")
    with open(folder.joinpath("frames.json"), "r") as f:
        old_step = 0
        first = True
        last = None
        with open(concatpath, "w+") as of:
            for line in f:
                frame = loads(line.strip())
                if not first:
                    of.write(f"duration {frame['step'] - old_step}" + "\n")
                file_line = f"file '{frame['path']}'" + "\n"
                of.write(file_line)
                old_step = frame["step"]
                first = False
                last = file_line
            of.write("duration 10\n" + last)

    cmd = [
        'ffmpeg',
        '-f', 'concat', '-i', str(concatpath),
        '-s', '1200x600', '-sws_flags', 'neighbor',
        '-vsync', 'vfr',
        '-c:v', 'libx264',
        '-vf', 'settb=AVTB,setpts=N/60/TB,fps=60',
        '-y', folder.joinpath("out.mkv")
    ]
    res = subprocess.call(cmd)
    assert res == 0


if __name__ == '__main__':
    main(Path("../output"))
