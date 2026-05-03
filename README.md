## MCTS for Checkers 

### Note 
This project was implemented by Carole Beaugeois and Angela Liu. We used the following git during the implementation : https://github.com/AngHengLiu/insa-4ir-ai-lab3-template

# How to use our project 
To run two AI engines, you have to complete two configurations in `main.rs`. You have three different engines : 
- RandomEngine that choses the next actions randomly between the valid actions, 
- MinimaxEngine that follows the Minimax algorithm, 
- MCTSEngine that implements the MCTS method. 

The `main.rs` computes 100 games with the same confugrations. To display correct measures taken during the execution, you must indicate in the `play_game` function the color that plays the MCTS engine. 

# Evaluation TO DO explain more when are the etrics from (every n moves ?)
To obtain a symetric evaluation for each configuration, we run 100 games with a certain color/role (White or Black), we exchange colors and we run the next 100 games. The displayed metrics are a mean of the 100 games and are from the engine with the color that you indicates in the `play_game` function in `main.rs`. 

## Evaluation with a baseline solver 

## Evaluation with another configuration of our solver 

### Weight of exploration 

It seems that the exploration weight is upside down ? 

### Evaluation function 
Presentation of evaluation function 



