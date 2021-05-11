# Generate a massive file for testing

dec = "I HAS A VALUE{0} ITZ \"HAI {0}\"\n"
assign = "VALUE{0} R SMOOSH VALUE{0} AN \"{0}\" MKAY\n"
visible = "VISIBLE VALUE{0}\n"

file = open("dec_assign_print.lol", "w+")
file.write("HAI 1.2\n")

for i in range(100000):
    file.write(dec.format(i))

for i in range(100000):
    file.write(assign.format(i))

for i in range(100000):
    file.write(visible.format(i))

file.write("KTHXBYE")
