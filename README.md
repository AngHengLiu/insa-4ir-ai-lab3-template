## MCTS for Checkers 

### Note 
This project was implemented by Carole Beaugeois and Angela Liu. We used the following git during the implementation : https://github.com/AngHengLiu/insa-4ir-ai-lab3-template

# How to use our project 
To run two AI engines, you have to complete two configurations in `main.rs`. You have three different engines : 
- RandomEngine that choses the next actions randomly between the valid actions, 
- MinimaxEngine that follows the Minimax algorithm, 
- MCTSEngine that implements the MCTS method. 

The `main.rs` computes 100 games with the same configurations. To display correct measures taken during the execution, you must indicate in the `play_game` function the color that plays the MCTS engine. 

# Evaluation TO DO explain more when are the etrics from (every n moves ?)
To obtain a symetric evaluation for each configuration, we run 100 games with a certain color/role (White or Black), we exchange colors and we run the next 100 games. The displayed metrics are a mean of the 100 games and are from the engine with the color that you indicates in the `play_game` function in `main.rs`. 

## Evaluation with a baseline solver 
For the absolute performance evaluation of the MCTS engine, we have compared it to the Minimax solver.

We have evaluated the MCTS engine by creating three different evaluations modifying the:
1. Time per move
2. Exploration weight of the MCTS
3. Evaluation function of the MCTS

The results can be found here: https://docs.google.com/spreadsheets/d/1C-duJAbnB5wOhUWz1Bo1mBrbsLdagbilawZmefNIsSo/edit?usp=sharing. They will be described and analysed below.

### Time per move
The goal is to analyse the effects of the given time per move for the engines on the result of a game. We try with 1ms up to 100ms. For each "time per move", we run 100 games, and when the MCTS engine is white and the Minimax engine is black and vice versa. For when White = MCTS and Black = Minimax, only the times per move that are in darker green are tested.

We can observe that, globally, as the time per move increases, the number of playouts per second increases. For when Black = MCTS, we can observe that White = Minimax globally wins less as the time per move increases. We think this is due to the fact that the MCTS, by having more time to do several playouts, can visit more nodes and select the node that is closer to the "optimal" one.

### Exploration weight 
Here, the goal is to analyse how changing the exploration weight for the MCTS engine will affects its performance.
We tested weights of 0.1 to 1, mostly with 20ms time per move, and sometimes with 10ms (in yellow), to see if that would change anything on the results. For each weight, we played 5 x 100 games to increase the preciceness of our results.

For when White = MCTS and Black = Minimax and 20ms time per move, we can observe that White wins more as the exploration weight increases from 0.1 up to and including 0.6. At some point around 0.8, the five results for weight 0.8 and 1 are significantly different from each other. We think that this is because the exploration term is favored "too" much, resulting in the MCTS engine exploring new nodes rather than expanding those already visited. This creates an imbalance that does not let the MCTS engine behave optimally.

## Evaluation with another configuration of our solver 

### Weight of exploration 

It seems that the exploration weight is upside down ? 

### Evaluation function 
Presentation of evaluation function 



