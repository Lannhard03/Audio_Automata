import pygame
import sys
import numpy as np
from scipy import signal
from scipy.ndimage import gaussian_filter


def update_automata(cells_1, cells_2, bass_intensity):
    # Update cells with a kernel based method?
    # Game of life: Compute neighbour sum using kernel
    neigh_ker_1 = np.array([[1, 1, 1], [1, 10, 1], [1, 1, 1]])
    neigh_ker_2 = np.array([[1, 1, 1], [1, 10, 1], [1, 1, 1]])
    sobel_y = np.array([[1, 2 , 0, -2 , -1],
                        [4, 8 , 0, -8 , -4],
                        [6, 12, 0, -12, -6],
                        [4, 8 , 0, -8 , -4],
                        [1, 2 , 0, -2 , -1]])
    sobel_x = np.transpose(sobel_y)

    neigh_1 = signal.convolve2d(cells_1, neigh_ker_1, mode='same', boundary='wrap')
    neigh_2 = signal.convolve2d(cells_2, neigh_ker_2, mode='same', boundary='wrap')

    #Idea: Cells 2 can attack, reproduce slightly less
    #For now bass_intensity is a value from 0-2
    #2 eats 1:
    attack = 4 - bass_intensity

    # If a cell1 is alive, and has too many cell2 neighbours, kill it.
    cells_1 = (neigh_1 >= 10)*(neigh_2 < attack)
    # Then it becomes a cell2
    cells_2 = cells_2 + (neigh_2 < 10)*(neigh_1 >= 10)*(neigh_2 >= attack)

    neigh_1 = signal.convolve2d(cells_1, neigh_ker_1, mode='same', boundary='wrap')
    neigh_2 = signal.convolve2d(cells_2, neigh_ker_2, mode='same', boundary='wrap')

    #Rules for 1:
    cells_1 = ((neigh_1 == 13) | (neigh_1 == 12)) + ((neigh_1 == 3) | (neigh_1 == 2))

    #Rules for 2:
    cells_2 = ((neigh_2 == 13) | (neigh_2 == 12)) + ((neigh_2 == 3))

    return (cells_1, cells_2)


def main():
    pygame.init()

    WIDHT, HEIGHT = 800, 800
    automata_width, automata_height = 256, 256
    frame_rate = 60
    screen = pygame.display.set_mode((WIDHT, HEIGHT))
    pygame.display.set_caption("Audio Automata")

    clock = pygame.time.Clock()
    running = True

    next_cells_1 = np.random.randint(0, 2, (automata_width, automata_height))
    next_cells_2 = np.random.randint(0, 2, (automata_width, automata_height))
    cells_1 = np.zeros((automata_width, automata_height))
    cells_2 = np.zeros((automata_width, automata_height))
    smooth_cells_1 = np.zeros((automata_width, automata_height))
    smooth_cells_2 = np.zeros((automata_width, automata_height))

    cell_update_rate = frame_rate/20

    current_frame = 0;
    # Main loop
    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            if event.type == pygame.MOUSEBUTTONDOWN:
                pos = pygame.mouse.get_pos()
                aut_x = int((pos[0]*automata_width) /WIDHT)
                aut_y = int((pos[1]*automata_height)/HEIGHT)
                cells_1[aut_x-3:aut_x+3, aut_y-3:aut_y+3] = np.ones((6,6))

        bass_intensity = 3*(np.sin(2*3.14*(current_frame/frame_rate)) > 0.9)
        if current_frame % cell_update_rate == 0:
            cells_1, next_cells_1 = next_cells_1, cells_1
            cells_2, next_cells_2 = next_cells_2, cells_2
            (next_cells_1, next_cells_2) = update_automata(cells_1, cells_2, bass_intensity)

        interp = ((current_frame % cell_update_rate)/cell_update_rate)
        interp_cells_1 = interp*next_cells_1 + (1-interp)*cells_1
        interp_cells_2 = interp*next_cells_2 + (1-interp)*cells_2

        gaussian_filter(interp_cells_1, sigma = 0, output = smooth_cells_1)
        gaussian_filter(interp_cells_2, sigma = 0, output = smooth_cells_2)

        #smooth_cells_1 = signal.convolve2d(cells_1, smooth_ker, mode='same', boundary='wrap')
        #smooth_cells_2 = signal.convolve2d(cells_2, smooth_ker, mode='same', boundary='wrap')

        rgb_arr = np.dstack([255*smooth_cells_2, 20*smooth_cells_1, 155*smooth_cells_1])
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
