# DesertDown Syntax Overview

---

## Paragraphs
Arguably the most important block - paragraphs. Paragraphs are a great place to use *italic text*, **bold text**, ***bold and italic text***, __underlined text__, ~~struck-through text~~, inline maths like $4x^2 + 3x + 2$, inline code like `print('Hello world!')`, inline comments that you can't see in the output%% super secret info!%%, text in ==not one highlight colour, ===not two highlight colours==, but three!===, tags like #desert-down and of course links: [[https://hfjn.co.uk]].

You could also embed an image: ![[cactus.svg]]

## Lists
- Some people love lists
  + I'd say I'm one of those people
                - You can indent them...
- ...or *not* indent them...
      * however you want!


## Callouts
Some things in life are important:
> [!important] Something very important
> Cacti are cool.

You can put whatever you like in a callout - even callouts:
> [!faq] Something about cacti
> Where, this callout says something else about cacti, but before I finish that, I have another thought:
> > [!danger] Cacti are dangerous!!!
> > **Don't** touch the spikes - they hurt :(

## Maths and Code
We should all spend a significant portion of our lives dedicated to thinking about the important things in life - music, coffee, cacti.

But sometimes, work is unavoidable. Math and code blocks can help here:

$$
\max_\underline{a} \min_\underline{x} \biggl\{ F(\underline{x}) +  \sum^N_{i=1} a_if_i(\underline{x}) \biggr\}
$$

```java
public class HelloWorld {
    public static void main(String[] args) {
        System.out.println("Hello world!");
    }
}
```

## Indent Blocks
Just like lists
        You can indent anything else too
        eg.
            $$
            x^2 + 1
            $$
        ---
        Even a break line can be indented.

And if you want to have a list:
- like this
  but prevent the next indent block getting captured by this list
  use a folding break `///` to get...
///
        This here, indented!!!
        

## Tables
Tables can have a header row, and text alignment within columns:

| Type of Cactus | Advantages | Disadvantages |
| :------------- | :--------: | ------------: |
| Tall | Instantly recognisable as a cactus | Vulnerable to falling over :( |
| Small | Much easier to store | Still needs watering |
| Dying | Spikes less sharp | Must water it NOW! |
| Dead | Doesn't need watering anymore | It's dead? |

## Blank lines
Most markdown parsers don't like to let you have thinking space. DesertDown does:







            ...







...Yay!


###### Fin!