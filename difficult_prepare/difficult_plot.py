import numpy as np
import matplotlib.pyplot as plt

level_time = 4 * 60

l = 20
s_d = 45.0 * 1000.0 / 3600.0
s_s = s_d * 0.5 * 0.2

print("Sheep time:", l / s_s)
print("Dog max travel time:", 2 * np.pi * l / s_d)

hardness = np.linspace(0.05, 1.0, 100)
k_h = 1.5

param_curve = l / s_s - 2 * np.pi * l / s_d - 1 / k_h / hardness

delta_t = 10
c = 10

n = (param_curve + delta_t) / np.sqrt(c)

plt.plot(hardness, n)
plt.grid()
plt.show()