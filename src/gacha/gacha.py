from flask import Flask, request, render_template
from flask_sqlalchemy import SQLAlchemy
import os

if os.environ['STATIC_PATH']:
    app = Flask(__name__, static_folder=os.environ['STATIC_PATH'])
else:
    app = Flask(__name__)
app.config['SQLALCHEMY_DATABASE_URI'] = os.environ['URI']
app.config['SQLALCHEMY_TRACK_MODIFICATIONS'] = False
db = SQLAlchemy(app)

from schema import *

prizes = [
    "Zoom Background",
    "Profile Picture",
    "Cyber Stickers",
    "Cyber Discord Role",
    "Cyber Discord Emote",
    "Cyber Serenade",
    "Steam Game"
]

@app.route('/rewards')
def rewards():
    team = Session.query.get(request.cookies.get('session')).team_lookup
    
    regular_tickets = (team.score - team.redeemed_score) // 50
    premium_tickets = team.premium_tickets

    return render_template(
        'rewards.html', 
        regular_tickets=regular_tickets, 
        premium_tickets=premium_tickets, 
        prizes=prizes)

@app.route('/rewards/redeem', methods=['POST'])
def redeem():
    reward_type = request.form.get("type")
    if reward_type is None:
        return "Error"

    return "hey"

if __name__ == '__main__':
    app.run()