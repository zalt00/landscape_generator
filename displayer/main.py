from direct.showbase.ShowBase import ShowBase
from panda3d.core import (Filename, HeightfieldTesselator, DirectionalLight,
                          AmbientLight, GeoMipTerrain, TextureStage, TexGenAttrib)
import math
from panda3d.core import Fog
import toml
import json
import datetime


class App(ShowBase):

    SB_SIZE_POW2 = 1
    SB_BASE_SIZE = 32.15

    SB_SIZE = SB_BASE_SIZE * 2**SB_SIZE_POW2

    
    def __init__(self):

        self.session_name = input("enter the name of the session: ")
        self.session_name += f"-{datetime.date.today()}"
        self.i = 0

        super().__init__()

        settings = toml.load("./settings.toml")

        self.camera.setPos(500, 500, 400)

        print("Creating terrain: ")
        self.terrain = GeoMipTerrain("main_terrain")

        print(" - loading heightmap...")
        self.terrain.setHeightfield("./heightmap.png")
        print(" - loading colormap...")

        tex = self.loader.loadTexture("./colormap.png")
        self.terrain.getRoot().setTexture(tex)
        
        self.terrain.setBruteforce(True)
        self.terrain.getRoot().setSz(settings["generation_options"]["max_terrain_height"])
        self.terrain.getRoot().reparentTo(self.render)
        print(" - generating model...")
        self.terrain.generate()
        print("done.\n")

        
        self.lighting_hpr = [0.0, settings["generation_options"]["sun_angle"], 0.0]

        print("Creating skysphere: ")
        self.sky_demisphere_terrain = GeoMipTerrain("sky_demisphere_terrain")

        print(" - loading heightmap...")
        self.sky_demisphere_terrain.setHeightfield("./demisphere_heightmap.png")
        print(" - loading colormap...")
        self.sky_demisphere_terrain.setColorMap("./demisphere_colormap.png")

        self.sky_demisphere_terrain.setBruteforce(True)

        self.root = self.render.attachNewNode('root')
        self.sky_demisphere_nodepath = self.sky_demisphere_terrain.getRoot()
        
        self.sky_demisphere_nodepath.setSz(self.SB_SIZE)
        self.sky_demisphere_nodepath.reparentTo(self.root)
        self.sky_demisphere_nodepath.setTwoSided(True)
        self.sky_demisphere_nodepath.setBin('background', 0)
        self.sky_demisphere_nodepath.setPos(-self.SB_SIZE, -self.SB_SIZE, 0.0)
        self.sky_demisphere_nodepath.setDepthWrite(0)

        self.root.setHpr(0.0 + self.lighting_hpr[0], 0.0 + self.lighting_hpr[1] + 90, 0.0 + self.lighting_hpr[2])

        print(" - generating model...")
        #self.sky_demisphere_terrain.generate()

        

        self.sky_demisphere_terrain_2 = GeoMipTerrain("sky_demisphere_terrain_2")

        print(" - copying to second demisphere")
        self.sky_demisphere_terrain_2.setHeightfield(self.sky_demisphere_terrain.heightfield())
        #self.sky_demisphere_terrain_2.setColorMap(self.sky_demisphere_terrain.colorMap())

        self.sky_demisphere_terrain_2.setBruteforce(True)

        self.sky_demisphere_nodepath_2 = self.sky_demisphere_terrain_2.getRoot()
        
        self.sky_demisphere_nodepath_2.setSz(self.SB_SIZE)
        self.sky_demisphere_nodepath_2.reparentTo(self.root)
        self.sky_demisphere_nodepath_2.setTwoSided(True)
        self.sky_demisphere_nodepath_2.setBin('background', 0)
        self.sky_demisphere_nodepath_2.setPos(+self.SB_SIZE, -self.SB_SIZE, 0.0)
        self.sky_demisphere_nodepath_2.setDepthWrite(0)
        tex = self.loader.loadTexture("./demisphere_colormap.png")
        print(tex.format)
        tex.format = 29
        self.sky_demisphere_nodepath_2.setTexture(tex)

        self.sky_demisphere_nodepath_2.setHpr(180.0, 180.0, 0.0)
        print(" - generating model...")
        self.sky_demisphere_terrain_2.generate()
        print("done.")
        
        self.fog = Fog("base fog")
        self.fog.setMode(Fog.MLinear)
        self.terrain.getRoot().setFog(self.fog)
        self.fog.setExpDensity(0.0005)
        
        # self.fog.setColor((0.0126532465, 0.23597072, 0.5843365, 0.7))
        self.fog.setColor((1.0, 1.0, 1.0, 1.0))
        
        self.gameTask = taskMgr.add(self.gameLoop, "gameLoop")

        self.keys = {
            "move_left": 0,
            "move_right": 0,
            "move_forward": 0,
            "move_backward": 0,
            "move_up": 0,
            "move_down": 0,
            "look_left": 0,
            "look_right": 0,
            "look_down": 0,
            "look_up": 0,
            "detach_camera": 0,
            "move_sky_up": 0,
            "move_sky_down": 0,
            "turn_sky_left": 0,
            "turn_sky_right": 0,
            "lock_sky_height": 0,
            "reload_textures": 0
            }

        self.dlight = DirectionalLight('my dlight')
        self.dlnp = self.render.attachNewNode(self.dlight)
        self.dlnp.setHpr(*self.lighting_hpr)
        self.dlnp.setColor((3.0, 3.0, 3.0, 0.8))

        self.alight = AmbientLight("ambient")
        self.alight.setColor((0.8, 0.8, 0.8, 0.0))
        self.alnp = self.render.attachNewNode(self.alight)
        # self.render.setLight(self.alnp)
        # self.render.setLight(self.dlnp)



        self.sky_demisphere_nodepath.setLightOff()
        self.sky_demisphere_nodepath_2.setLightOff()

        
        self.hpr = [0.0, 0.0, 0.0]
        self.xyz = [0.0, 0.0, 0.0]

        self.sky_height = 0.0
        self.sky_heading = 0.0
        
        self.accept("space", self.setKey, ["move_up", 1])
        self.accept("space-up", self.setKey, ["move_up", 0])
        self.accept("control", self.setKey, ["move_down", 1])
        self.accept("control-up", self.setKey, ["move_down", 0])

        self.accept("k", self.setKey, ["look_left", 1])
        self.accept("k-up", self.setKey, ["look_left", 0])
        self.accept("m", self.setKey, ["look_right", 1])
        self.accept("m-up", self.setKey, ["look_right", 0])
        self.accept("o", self.setKey, ["look_up", 1])
        self.accept("o-up", self.setKey, ["look_up", 0])
        self.accept("l", self.setKey, ["look_down", 1])
        self.accept("l-up", self.setKey, ["look_down", 0])

        self.accept("z", self.setKey, ["move_forward", 1])
        self.accept("z-up", self.setKey, ["move_forward", 0])
        self.accept("s", self.setKey, ["move_backward", 1])
        self.accept("s-up", self.setKey, ["move_backward", 0])
        self.accept("q", self.setKey, ["move_left", 1])
        self.accept("q-up", self.setKey, ["move_left", 0])
        self.accept("d", self.setKey, ["move_right", 1])
        self.accept("d-up", self.setKey, ["move_right", 0])

        self.accept("b", self.setKey, ["detach_camera", 1])
        self.accept("n", self.setKey, ["detach_camera", 0])

        self.accept("t", self.setKey, ["move_sky_up", 1])
        self.accept("t-up", self.setKey, ["move_sky_up", 0])

        self.accept("g", self.setKey, ["move_sky_down", 1])
        self.accept("g-up", self.setKey, ["move_sky_down", 0])        

        self.accept("f", self.setKey, ["turn_sky_left", 1])
        self.accept("f-up", self.setKey, ["turn_sky_left", 0])

        self.accept("h", self.setKey, ["turn_sky_right", 1])
        self.accept("h-up", self.setKey, ["turn_sky_right", 0])

        self.accept("a", self.setKey, ["lock_sky_height", 1])
        self.accept("a-up", self.setKey, ["lock_sky_height", 0])

        self.accept("w", self.reload_textures)

        self.accept("v", self.take_screenshot)

        self.accept("x", self.save_settings)
        self.accept("c", self.load_settings)

    def take_screenshot(self):
        name = f"screenshots/{self.session_name}-{self.i}.png"
        base.screenshot(name, False)

        self.i += 1

        print(f"Saved screenshot as name {name}")


    def save_settings(self):
        with open("displayer/settings.json", "w") as file:
            json.dump(dict(camera_hpr=self.hpr, camera_xyz=self.xyz, sky_height=self.sky_height, sky_rotation=self.lighting_hpr[0]), file)
        print("config saved")

    def load_settings(self):
        with open("displayer/settings.json") as file:
            data = json.load(file)

        self.hpr[:] = data["camera_hpr"]
        self.xyz[:] = data["camera_xyz"]
        self.sky_height = data["sky_height"]
        self.lighting_hpr[0] = data["sky_rotation"]

        print("config loaded")

    def reload_textures(self):
        print(0)
        tex = self.loader.loadTexture("./demisphere_colormap.png")
        self.sky_demisphere_nodepath_2.setTexture(tex)

        tex = self.loader.loadTexture("./colormap.png")
        self.terrain.getRoot().setTexture(tex)
        print(1)
        
    def setKey(self, key, val):
        self.keys[key] = val

    def gameLoop(self, task):
        dt = globalClock.getDt()

        if self.keys["move_up"]:
            self.xyz[2] += 100.0 * dt
        elif self.keys["move_down"]:
            self.xyz[2] -= 100.0 * dt

        if self.keys["look_left"]:
            self.hpr[0] += 60 * dt
        elif self.keys["look_right"]:
            self.hpr[0] -= 60 * dt
        elif self.keys["look_down"]:
            self.hpr[1] -= 40 * dt
        elif self.keys["look_up"]:
            self.hpr[1] += 40 * dt


        if self.keys["move_forward"]:
            self.xyz[0] += math.cos((self.hpr[0] + 90) * math.pi / 180) * 100 * dt
            self.xyz[1] += math.sin((self.hpr[0] + 90) * math.pi / 180) * 100 * dt
        elif self.keys["move_backward"]:
            self.xyz[0] -= math.cos((self.hpr[0] + 90) * math.pi / 180) * 100 * dt
            self.xyz[1] -= math.sin((self.hpr[0] + 90) * math.pi / 180) * 100 * dt
        elif self.keys["move_right"]:
            self.xyz[0] += math.cos((self.hpr[0] + 0) * math.pi / 180) * 100 * dt
            self.xyz[1] += math.sin((self.hpr[0] + 0) * math.pi / 180) * 100 * dt
        elif self.keys["move_left"]:
            
            self.xyz[0] -= math.cos((self.hpr[0] + 0) * math.pi / 180) * 100 * dt
            self.xyz[1] -= math.sin((self.hpr[0] + 0) * math.pi / 180) * 100 * dt


        if self.keys["move_sky_up"]:
            self.sky_height += 40 * dt
        elif self.keys["move_sky_down"]:
            self.sky_height -= 40 * dt

        if self.keys["turn_sky_left"]:
            self.lighting_hpr[0] += 60 * dt
        elif self.keys["turn_sky_right"]:
            self.lighting_hpr[0] -= 60 * dt

        if self.keys["lock_sky_height"]:
            self.sky_height = self.xyz[2]
        
        if not self.keys["detach_camera"]:
            self.root.setPos(self.xyz[0], self.xyz[1], self.sky_height)
            self.root.setHpr(180.0 + self.lighting_hpr[0], 0.0 + self.lighting_hpr[1] + 90, 0.0 + self.lighting_hpr[2])

        self.dlnp.setHpr(self.lighting_hpr[0], 90 - self.lighting_hpr[1], self.lighting_hpr[2])
        
        self.camera.setPos(*self.xyz)

        self.camera.setHpr(*self.hpr)
        
        return task.cont    # Since every return is Task.cont, the task will

    
    def test(self):
        print("keyDown")
        
app = App()
app.run()
