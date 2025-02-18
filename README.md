# Command line tool to track your habits.

*Built to use with [Pixe.la](https://pixe.la/), works best with premium.*
## Installation 
Pull and build with cargo.

```cargo build --release```

## Set up
If you have a pixela account:
``` login <upixela username> <pixela api key/token> ```
Your data will be stored in a private local database.
If you don't have a pixela account you can create one using:
``` signup <your api key/token> <desired username> <Agree to TOS [yes/no]> <Are you a minor? [yes/no]> ```

## Usage
You can use a selection of commands to manipulate your pixela graphs.
```
data       Print your data
create     Creates a new graph on Pixela
send       Use to send pixels to Pixela
get        Use to get pixels data from Pixela
list       List all graphs
streak     Calculates your current streak of consecutive pixels
setup-sum  Setup sum graph functionality
sum        Sums today progress of your "graphs to sum" and uploads it to sum graph
```

### Sum graphs (w.i.p)
As of right now you can specify two graphs of which the progress will be summarized to a third graph. 
You set it all up using ``` setup-sum ``` then ``` sum ``` to push to the sum graph (third graph you specified).

