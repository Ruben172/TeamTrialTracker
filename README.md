# TeamTrialTracker

TeamTrialTracker is an OCR application for Umamusume: Pretty Derby. 
This application reads screenshots of your Team Trials scores, to render them in a boxplot. 
This aims to help players compare their uma's performances, to know which umas are inconsistent or need work.

# Example
<table>
  <tr>
    <td><img src="https://github.com/user-attachments/assets/1a86a91f-bf71-4146-a925-4078a10260f7" alt="Boxplot showing Team Trials scores sorted by minimum score" /><p align="center">Scores sorted by minimum</p></td>
    <td><img src="https://github.com/user-attachments/assets/3000ea3a-9822-4b96-8db7-dd1ff680184c" alt="Boxplot showing Team Trials scores sorted by median score" /><p align="center">Scores sorted by median</p></td>
  </tr>
  <tr>
    <td><img src="https://github.com/user-attachments/assets/3bae24d4-4a0b-424b-8213-8ffe7cbffe99" alt="Boxplot showing Team Trials scores sorted by mean score" /><p align="center">Scores sorted by mean</p></td>
    <td><img src="https://github.com/user-attachments/assets/763affd8-2e88-43a1-b7e7-f0432089f5cf" alt="Boxplot showing Team Trials scores sorted by maximum score" /><p align="center">Scores sorted by maximum</p></td>
  </tr>
</table>

# Installation
Download the latest [release](https://github.com/Ruben172/TeamTrialTracker/releases) from GitHub, extract the ZIP from where you want to run the program.

You also need [Firefox](https://firefox.com/) installed for the boxplots to render.

# Usage
Currently, specific cropping regions are set to make scanning the screenshots most accurate. It is thus advised to always take full-screen screenshots, as it will default to scanning the whole screenshot otherwise (which might lead to missed data).

- For every score you wish to import, take one screenshot when scrolled to the top, and one when scrolled to the bottom. There should be no duplicate scores on your screenshots.
- Move the screenshots to the included `/input` folder.
- Run `TeamTrialTracker.exe`. The program will now read your screenshots and save a `scores.json` as well as four boxplots in an `/output` folder.

## Removing scores
To remove scores, please open `output/scores.json` in a text editor and remove the scores manually. Make sure the JSON remains properly formatted (there should be no trailing comma after the last score, and all brackets need to be closed properly)

# Troubleshooting
If something is not working, running the program from a terminal should show you an error. Most common causes of errors are either Firefox not being installed, or the Geckodriver being incorrectly closed. Check if `geckodriver` is running on your system, and kill it manually if it is.

If something is not working as intended, please open an [issue](https://github.com/Ruben172/TeamTrialTracker/issues) on GitHub or message me on discord `@koish1`.

# Planned features
- Write a custom boxplot renderer to not need a browser anymore.
- Better min/max score sorting based on whiskers instead of the lowest/highest score.
- Progress bars.

# Credits
- Uses [ocrs](https://crates.io/crates/ocrs) for reading scores.
- Uses [plotly](https://crates.io/crates/plotly/0.8.4) for drawing the boxplots.
- [osu! Game](https://discord.gg/osu) uma community for beta testing.


This project includes geckodriver, licensed under the Mozilla Public License 2.0.
Source: https://github.com/mozilla/geckodriver
