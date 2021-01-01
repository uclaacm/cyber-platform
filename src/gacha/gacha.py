from flask import Flask, request, render_template
from flask_sqlalchemy import SQLAlchemy
import os
import random
from flask_wtf.csrf import CSRFProtect

if os.environ['STATIC_PATH']:
    app = Flask(__name__, static_folder=os.environ['STATIC_PATH'])
else:
    app = Flask(__name__)
app.config['SQLALCHEMY_DATABASE_URI'] = os.environ['URI']
app.config['SECRET_KEY'] = os.environ['SECRET_KEY']
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

one_time_only = [
    ('Cyber Stickers', 2.0),
    ('Cyber Discord Role', 30.0),
    ('Cyber Discord Emote', 0.5),
    ('Cyber Serenade', 0.5),
    ('Steam Game', 0.5),
]

@app.route('/rewards')
def rewards():
    team = Session.query.get(request.cookies.get('session'))
    if team is None:
        return render_template('error.html', message="Login to view rewards.")
    team = team.team_lookup
    
    regular_tickets = (team.score - team.redeemed_score) // 50
    premium_tickets = team.premium_tickets

    return render_template(
        'rewards.html', 
        regular_tickets=regular_tickets, 
        premium_tickets=premium_tickets, 
        prizes=prizes,
        logged_in=True)

@app.route('/rewards/redeem', methods=['POST'])
def redeem():
    reward_type = request.form.get("type")
    if reward_type is None:
        return "Invalid request", 400

    team = Session.query.get(request.cookies.get('session'))
    if team is None:
        return render_template('error.html', message="Login to view rewards.")
    team = team.team_lookup

    regular_tickets = (team.score - team.redeemed_score) // 50
    premium_tickets = team.premium_tickets

    if reward_type == 'regular':
        if regular_tickets > 0:
            one_time_weights = []
            prize_exists = {
                'Zoom Background': True,
                'Profile Picture': True
            }

            for prize, prob in one_time_only:
                exists = Prize.query.filter_by(team=team.id, prize=prize).first()
                prize_exists[prize] = not(exists is None)
                if exists:
                    one_time_weights.append(0)
                else:
                    one_time_weights.append(prob)
            leftover_prob = 100.0 - sum(one_time_weights)
            zoom = 0.6 * leftover_prob
            profile = 0.4 * leftover_prob
            weights = [zoom, profile] + one_time_weights
            prize = random.choices(prizes, weights=weights)[0]

            team.redeemed_score = team.redeemed_score + 50
            db.session.commit()

            if not prize_exists[prize]:
                prize_record = Prize(team.id, prize)
                db.session.add(prize_record)
                db.session.commit()

            return render_template('redeem.html', prize=prize, enough=True, regular=True, logged_in=True)
        else:
            return render_template('redeem.html', enough=False, logged_in=True)
    elif reward_type == 'premium':
        if premium_tickets > 0:
            team.premium_tickets = premium_tickets - 1
            db.session.commit()

            raffle = Raffle(team.id)
            db.session.add(raffle)
            db.session.commit()

            return render_template('redeem.html', enough=True, regular=False, logged_in=True)
        else:
            return render_template('redeem.html', enough=False, logged_in=True)
    else:
        return "Invalid request", 400

if __name__ == '__main__':
    app.run()