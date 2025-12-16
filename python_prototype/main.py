import pygame
import sys
import numpy as np
from scipy import signal
from scipy.ndimage import gaussian_filter
import sounddevice as sd
import soundfile as sf
import threading
import queue

class Automata:
    def __init__(self, width, height):
        self.alive_threshold = 0.5

        self.overpop_threshold = 6
        self.overpop_dead_prm = 0
        self.overpop_alive_prm = 0

        self.repop_min = 6
        self.repop_max = 6
        self.repop_dead_prm = 0
        self.repop_alive_prm = 0

        self.lonliness_threshhold = 0
        self.lonliness_prm = 0

        self.starvation_prm = 0

        self.next_cells = 1/100*np.random.randint(0, 100, (width, height))
        self.cells = np.zeros((width, height))

        self.neigh_ker = np.array([[1, 1, 1], [1, 1, 1], [1, 1, 1]])
        self.update_cell_states()


    def update_cell_states(self):
        np.clip(self.next_cells, 0, 1, out=self.next_cells)
        self.alive = (self.next_cells >= self.alive_threshold)
        self.neigh = signal.convolve2d(self.next_cells, self.neigh_ker, mode='same', boundary='wrap')


    def update_cells(self):
        self.next_cells = self.cells

        #Overpopulation
        overpopulated = (self.neigh >= self.overpop_threshold)
        self.next_cells -= (self.alive)*overpopulated*(self.overpop_alive_prm*self.neigh)
        self.next_cells -= (np.invert(self.alive))*overpopulated*(self.overpop_dead_prm*self.neigh)

        #Reproduction
        repop = (self.neigh >= self.repop_min)*(self.neigh <= self.repop_max)
        self.next_cells += (self.alive)*repop*(self.repop_alive_prm*self.neigh)
        self.next_cells += (np.invert(self.alive))*repop*(self.repop_dead_prm*self.neigh)

        #Lonliness
        self.next_cells -= self.lonliness_prm*(self.alive)*(self.neigh <= self.lonliness_threshhold)

        #Starvation
        self.next_cells -= self.starvation_prm


        self.update_cell_states()
        self.cells, self.next_cells = self.next_cells, self.cells

#Modifiers
class PredatorPrey:
    def __init__(self, predator, prey):
        self.pred = predator
        self.prey = prey
        self.eat_threshold = 0.9
        self.gain_parameter = 0.05
        self.loss_parameter = 0.2

    def apply(self):
        eating = self.pred.neigh * self.prey.neigh * (self.prey.neigh >= self.eat_threshold) 
        self.pred.next_cells += self.gain_parameter*eating
        self.prey.next_cells -= self.loss_parameter*eating

        self.pred.update_cell_states()
        self.prey.update_cell_states()


def handle_events(WIDHT, HEIGHT, automata_width, automata_height, aut_1, aut_2):
    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            return False
        if event.type == pygame.MOUSEBUTTONDOWN:
            pos = pygame.mouse.get_pos()
            aut_x = int((pos[0]*automata_width) / WIDHT)
            aut_y = int((pos[1]*automata_height) / HEIGHT)
            aut_2.next_cells[aut_x-3:aut_x+3, aut_y-3:aut_y+3] = np.ones((6, 6))
        if event.type == pygame.KEYDOWN and event.key == pygame.K_SPACE:
            pos = pygame.mouse.get_pos()
            aut_x = int((pos[0]*automata_width) / WIDHT)
            aut_y = int((pos[1]*automata_height) / HEIGHT)
            aut_1.next_cells[aut_x-3:aut_x+3, aut_y-3:aut_y+3] = np.ones((6, 6))

    return True



playback_frame = 0
def main():
    pygame.init()
    WIDHT, HEIGHT = 800, 800
    automata_width, automata_height = 256, 256

    frame_rate = 60
    screen = pygame.display.set_mode((WIDHT, HEIGHT))
    pygame.display.set_caption("Audio Automata")

    DEVICE = 5      # change to your BlackHole device index
    SAMPLE_RATE = 48000
    BLOCK_SIZE = 1024 

    clock = pygame.time.Clock()
    running = True

    aut_1 = Automata(automata_width, automata_height)
    aut_1.alive_threshold = 1/2

    aut_1.overpop_threshold = 4
    aut_1.overpop_dead_prm = 1/10
    aut_1.overpop_alive_prm = 1/15

    aut_1.repop_min = 3
    aut_1.repop_max = 5
    aut_1.repop_dead_prm = 6/6
    aut_1.repop_alive_prm = 1/3

    aut_1.lonliness_threshhold = 10
    aut_1.lonliness_prm = 1/2

    aut_1.starvation_prm = 1/10


    aut_2 = Automata(automata_width, automata_height)
    aut_2.alive_threshold = 1/2

    aut_2.overpop_threshold = 4
    aut_2.overpop_dead_prm = 1/6
    aut_2.overpop_alive_prm = 1/10

    aut_2.repop_min = 2
    aut_2.repop_max = 3
    aut_2.repop_dead_prm = 0.2
    aut_2.repop_alive_prm = 0.1

    aut_2.lonliness_threshhold = 10
    aut_2.lonliness_prm = 1/3

    aut_2.starvation_prm = 1/24

    pred_prey = PredatorPrey(aut_2, aut_1)
    pred_prey.gain_parameter = 0.025
    pred_prey.loss_parameter = 0.05


    aut_2.next_cells = aut_2.cells #Hack to set next_cells to zero
    aut_2.update_cell_states()
    aut_1.next_cells = aut_1.cells #Hack to set next_cells to zero
    aut_1.update_cell_states()

    smooth_cells_1 = np.zeros((automata_width, automata_height))
    smooth_cells_2 = np.zeros((automata_width, automata_height))

    cell_update_rate = frame_rate/20

    data, fs = sf.read("./test_music/Altinbas - Good Intentions.mp3", always_2d=True)
    event = threading.Event()
    audio_q = queue.Queue(maxsize=16)
    playback_frame = 0
    bpm = 138
    bpf = (bpm/60)/frame_rate
    beat_sum = 0

    def callback(outdata, frames, time, status):
        global playback_frame
        if status:
            print(status)
        chunksize = min(len(data) - playback_frame, frames)
        outdata[:chunksize] = data[playback_frame:playback_frame + chunksize]
        # Send to analysis queue
        try:
            audio_q.put_nowait(outdata[:chunksize].copy())
        except queue.Full:
            pass
        if chunksize < frames:
            outdata[chunksize:] = 0
            raise sd.CallbackStop()
        playback_frame += chunksize

    stream = sd.OutputStream(
        samplerate=fs, device=2, channels=data.shape[1],
        callback=callback, finished_callback=event.set)

    with stream:
        #event.wait()

        current_frame = 0
        # Main loop
        while running:
            beat_sum += bpf
            if beat_sum >= 1:
                beat_sum = 0
                print("beat")
            up_beat = (beat_sum >= 0.8)


            running = handle_events(WIDHT, HEIGHT, automata_width, automata_height, aut_1, aut_2)

            if not audio_q.empty():
                block = audio_q.get()
                mono = block.mean(axis=1)
                freq = np.fft.rfft(mono)
                bass_freq_cutoff = 100
                bass_energy = np.sqrt(np.mean(np.abs(freq[0:bass_freq_cutoff])**2))
                pred_prey.gain_parameter = 0.0175 + up_beat*(100)
                aut_1.overpop_alive_prm = 1/15 + (bass_energy/30)**4
                print("bass energy:", bass_energy)


            if current_frame % cell_update_rate == 0:
                pred_prey.apply()
                aut_1.update_cells()
                aut_2.update_cells()




            # Draw cells
            interp = ((current_frame % cell_update_rate)/cell_update_rate)
            interp_cells_1 = interp*aut_1.next_cells + (1-interp)*aut_1.cells
            interp_cells_2 = interp*aut_2.next_cells + (1-interp)*aut_2.cells

            gaussian_filter(interp_cells_1, sigma = 0, output = smooth_cells_1)
            gaussian_filter(interp_cells_2, sigma = 0, output = smooth_cells_2)

            rgb_arr = np.dstack([(200 + 50*up_beat)*smooth_cells_2, 50*smooth_cells_1, (155)*smooth_cells_1])
            rgb_surf = pygame.surfarray.make_surface(rgb_arr)

            screen.fill((0, 0, 0))
            screen.blit(
                pygame.transform.scale(rgb_surf, (WIDHT, HEIGHT)), (0, 0)
            )
            pygame.display.flip()

            clock.tick(frame_rate)
            current_frame += 1

        pygame.quit()
        sys.exit()


if __name__ == "__main__":
    main()
