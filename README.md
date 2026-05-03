## MCTS for Checkers 

### Note 
This project was implemented by Carole Beaugeois and Angela Liu. We used the following git during the implementation : https://github.com/AngHengLiu/insa-4ir-ai-lab3-template

# How to use our project 
To run two AI engines, you have to complete two configurations in `main.rs`. You have three different engines : 
- RandomEngine that choses the next actions randomly between the valid actions, 
- MinimaxEngine that follows the Minimax algorithm, 
- MCTSEngine that implements the MCTS method. 

The `main.rs` computes 100 games with the same confugrations. To display correct measures taken during the execution, you must indicate in the `play_game` function the color that plays the MCTS engine. 

# Evaluation 
To obtain a symetric evaluation for each configuration, we run 100 games with a certain color/role (White or Black), we exchange colors and we run the next 100 games. The displayed metrics are a mean of the 100 games and are from the engine with the color that you indicates in the `play_game` function in `main.rs`. 

## Evaluation with a baseline solver 

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

As the previous method, our goal is to focus only on the parameter we want and to let other values untouched. 

The most obvious result is that the heuristic evaluation is way less effective than a rollout (100 defeats as Black and 99 defeats as White). 

For the rollout function, a larger number of rollouts win against a 


