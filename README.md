## MCTS for Checkers 

### Note 
This project was implemented by Carole Beaugeois and Angela Liu. We used the following git during the implementation : https://github.com/AngHengLiu/insa-4ir-ai-lab3-template

# How to use our project 
To run two AI engines, you have to complete two configurations in `main.rs`. You have three different engines : 
- RandomEngine that choses the next actions randomly between the valid actions, 
- MinimaxEngine that follows the Minimax algorithm, 
- MCTSEngine that implements the MCTS method. 

The `main.rs` computes 100 games with the same configurations. To display correct measures taken during the execution, you must indicate in the `play_game` function the color that plays the MCTS engine. 

# Evaluation 
To obtain a symetric evaluation for each configuration, we run 100 games with a certain color/role (White or Black), we exchange colors and we run the next 100 games. The displayed metrics are a mean of the 100 games and are from the engine with the color that you indicates in the `play_game` function in `main.rs`. 

## Evaluation with a baseline solver 
For the absolute performance evaluation of the MCTS engine, we have compared it to the Minimax solver.

We have evaluated the MCTS engine by creating two different evaluations modifying the time per move and the exploration weight of the MCTS engine.

The results can be found here: https://docs.google.com/spreadsheets/d/1C-duJAbnB5wOhUWz1Bo1mBrbsLdagbilawZmefNIsSo/edit?usp=sharing. They will be described and analysed below.

### Time per move
The goal is to analyse the effects of the given time per move for the engines on the result of a game. We try with 1ms up to 100ms. For each "time per move", we run 100 games, and when the MCTS engine is white and the Minimax engine is black and vice versa. For when White = MCTS and Black = Minimax, only the times per move that are in darker green are tested.

We can observe that, globally, as the time per move increases, the number of playouts per second increases. For when Black = MCTS, we can observe that White = Minimax globally wins less as the time per move increases. We think this is due to the fact that the MCTS, by having more time to do several playouts, can visit more nodes and select the node that is closer to the "optimal" one.

### Exploration weight 
Here, the goal is to analyse how changing the exploration weight for the MCTS engine will affects its performance.
We tested weights of 0.1 to 1, mostly with 20ms time per move, and sometimes with 10ms (in yellow), to see if that would change anything on the results. For each weight, we played 5 x 100 games to increase the preciceness of our results.

For when White = MCTS and Black = Minimax and 20ms time per move, we can observe that White wins more as the exploration weight increases from 0.1 up to and including 0.6. At some point around 0.8, the five results for weight 0.8 and 1 are significantly different from each other. We think that this is because the exploration term is favored "too" much, resulting in the MCTS engine exploring new nodes rather than expanding those already visited. This creates an imbalance that does not let the MCTS engine behave optimally.

## Evaluation with another configuration of our solver 
For this type of evaluation, we focus on two main parameters to compete different configuration of MCTS engines against each other : exploration weight and evalation function. 
All the evaluations are available here : https://docs.google.com/spreadsheets/d/1ZaC4xy4BpxbXvcSGgWrhrqOCkenMaSYpWdeP0Gwbjwo/edit?usp=sharing .
For each duel (100 games as White and 100 games as Black), the results of the winner are green. 

### Exploration weight C
The goal is to evaluate the impact of the exploration weight that is between 0.0 and 1.0. To do so, we only change that value during our evaluations (we change the time per move one time to obtain a winner because one evaluation doesn't clearly select a winner). 
We notice that the higher exploration weight always win and the winner also has the lower number of playouts per second. This number discreases during the game because the graph that contains all the nodes growths during the game and the discovery of new nodes. It then takes more time to reach undiscoverd nodes and to evaluate them at the end of the game than at the beginning. 

We also note that the depth of the playouts increases with the exploration weight. We see a depth higher than 4.5 for weight greater or equal to 0.9. For weight lower than 0.5, the depth doesn't exceed 3.0. 
That result is surprising because with a high exploration weight, we should explore more branchs of our graph and not exploit a lot existing branch to go deeper. We conclude that our algorith may not perform correctly on the selection part.   

### Evaluation function 
As suggested, we add a new evaluation function to replace the rollout. We use the `heuristic_evaluation` that is used by the Minimax engine. To use it, we implement a new parameter `eval_function` when creating a MctsEngine. 0 corresponds to the rollout function and 1 to the heuristic evaluation. We enable the possibility to do a mean of rollouts to evaluate a new node. To select how many rollouts we want for our mean, MctsEngine has another new parameter : `value_eval`. 

As the previous method, our goal is to focus only on the parameter we want and to let other values untouched. We fix an arbitrary exploration weight to 0.3.  

The most obvious result is that the heuristic evaluation is way less effective than a rollout (100 defeats as Black and 99 defeats as White). 

For the rollout function, a larger number of rollouts win against a smaller number. However, that trend stops between 30 and 50 rollouts for 20 ms per move. We guess that our engine doesn't have enough time to do its rollouts, leading to defeats. When we increased that time to 100 ms, our engine with 50 rollouts win against our engine with 30 rollouts. 

To compare the impact of our exploration weight against the number of rollouts, we did some evaluations with an engine with an exploration weight of 1.0 and a fewer rollouts than its opponent. We didn't study a lot that case but we notice that the engine with a larger number of rollouts win with less ease or even loses depending on the different configurations. 

To focus on the metrics, the number of playouts per second decreases when we increase the number of rollouts. We expected that behavior because we spend more time to do rollouts so there are less time left to do a lot of playouts. For playout depths, the greater values are obtained with an exploration weight with the value 1.0. It is consistent with our previous results.   
