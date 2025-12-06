import pygame
import sys
import numpy as np
from scipy import signal
from scipy.ndimage import gaussian_filter
import sounddevice as sd


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
        self.eat_threshold = 0.5
        self.gain_parameter = 0.05
        self.loss_parameter = 0.2

    def apply(self):
        eating = self.pred.neigh * self.prey.neigh * (self.prey.neigh >= self.eat_threshold) 
        self.pred.next_cells += self.gain_parameter*eating
        self.prey.next_cells -= self.loss_parameter*eating

        self.pred.update_cell_states()
        self.prey.update_cell_states()


def main():
    pygame.init()
    WIDHT, HEIGHT = 800, 800
    automata_width, automata_height = 256, 256
    frame_rate = 60
    screen = pygame.display.set_mode((WIDHT, HEIGHT))
    pygame.display.set_caption("Audio Automata")

    clock = pygame.time.Clock()
    running = True

    aut_1 = Automata(automata_width, automata_height)
    aut_1.alive_threshold = 1/2

    aut_1.overpop_threshold = 6
    aut_1.overpop_dead_prm = 1/6
    aut_1.overpop_alive_prm = 1/10

    aut_1.repop_min = 3
    aut_1.repop_max = 5
    aut_1.repop_dead_prm = 5/6
    aut_1.repop_alive_prm = 1/3

    aut_1.lonliness_threshhold = 10
    aut_1.lonliness_prm = 1/2

    aut_1.starvation_prm = 0


    aut_2 = Automata(automata_width, automata_height)
    aut_2.alive_threshold = 1/2

    aut_2.overpop_threshold = 3
    aut_2.overpop_dead_prm = 1/6
    aut_2.overpop_alive_prm = 1/10

    aut_2.repop_min = 2
    aut_2.repop_max = 3
    aut_2.repop_dead_prm = 0.2
    aut_2.repop_alive_prm = 0.1

    aut_2.lonliness_threshhold = 10
    aut_2.lonliness_prm = 1/2

    aut_2.starvation_prm = 1/16

    pred_prey = PredatorPrey(aut_2, aut_1)

    aut_2.next_cells = aut_2.cells #Hack to set next_cells to zero
    aut_2.update_cell_states()

    smooth_cells_1 = np.zeros((automata_width, automata_height))
    smooth_cells_2 = np.zeros((automata_width, automata_height))

    cell_update_rate = frame_rate/20

    current_frame = 0
    # Main loop
    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            if event.type == pygame.MOUSEBUTTONDOWN:
                pos = pygame.mouse.get_pos()
                aut_x = int((pos[0]*automata_width) / WIDHT)
                aut_y = int((pos[1]*automata_height) / HEIGHT)
                aut_2.next_cells[aut_x-3:aut_x+3, aut_y-3:aut_y+3] = np.ones((6, 6))

        bass_intensity = 3*(np.sin(2*3.14*(current_frame/frame_rate)) > 0.9)

        if current_frame % cell_update_rate == 0:
            pred_prey.apply()
            aut_1.update_cells()
            aut_2.update_cells()


        interp = ((current_frame % cell_update_rate)/cell_update_rate)
        interp_cells_1 = interp*aut_1.next_cells + (1-interp)*aut_1.cells
        interp_cells_2 = interp*aut_2.next_cells + (1-interp)*aut_2.cells

        gaussian_filter(interp_cells_1, sigma = 0, output = smooth_cells_1)
        gaussian_filter(interp_cells_2, sigma = 0, output = smooth_cells_2)

        rgb_arr = np.dstack([200*smooth_cells_2, 50*smooth_cells_1, 155*smooth_cells_1])
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
