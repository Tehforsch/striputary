set terminal pngcairo
set output "pics/volumePlot.png"
set xlabel "Cut Offset [s]"
set ylabel "Av. Volume [s]"
plot "plot/data" u 2:3 w l ls 1 t ""