# Generated by Django 4.2.1 on 2023-08-26 11:41

from django.db import migrations, models
import django.db.models.deletion


class Migration(migrations.Migration):
    initial = True

    dependencies = []

    operations = [
        migrations.CreateModel(
            name="CustomUser",
            fields=[
                (
                    "id",
                    models.BigAutoField(
                        auto_created=True,
                        primary_key=True,
                        serialize=False,
                        verbose_name="ID",
                    ),
                ),
                ("created_on", models.DateTimeField(auto_now_add=True)),
                ("updated_on", models.DateTimeField(auto_now=True)),
                ("name", models.CharField(max_length=127, null=True)),
                ("phone", models.CharField(max_length=20, null=True, unique=True)),
                ("email", models.CharField(max_length=50, null=True, unique=True)),
                ("active", models.BooleanField(default=True)),
                ("last_login", models.DateTimeField(null=True)),
            ],
            options={
                "db_table": "authapp_user",
            },
        ),
        migrations.CreateModel(
            name="UserOtp",
            fields=[
                (
                    "id",
                    models.BigAutoField(
                        auto_created=True,
                        primary_key=True,
                        serialize=False,
                        verbose_name="ID",
                    ),
                ),
                ("created_on", models.DateTimeField(auto_now_add=True)),
                ("updated_on", models.DateTimeField(auto_now=True)),
                ("email", models.CharField(max_length=50, null=True, unique=True)),
                ("phone", models.CharField(max_length=20, null=True, unique=True)),
                ("otp_bucket", models.JSONField()),
                ("status", models.CharField(max_length=50)),
            ],
            options={
                "db_table": "authapp_user_otp",
            },
        ),
        migrations.CreateModel(
            name="UserToken",
            fields=[
                (
                    "id",
                    models.BigAutoField(
                        auto_created=True,
                        primary_key=True,
                        serialize=False,
                        verbose_name="ID",
                    ),
                ),
                ("created_on", models.DateTimeField(auto_now_add=True)),
                ("updated_on", models.DateTimeField(auto_now=True)),
                ("token", models.CharField(max_length=255)),
                ("active", models.BooleanField(default=True)),
                ("device_number", models.CharField(max_length=255, null=True)),
                (
                    "user",
                    models.ForeignKey(
                        on_delete=django.db.models.deletion.PROTECT,
                        to="authapp.customuser",
                    ),
                ),
            ],
            options={
                "db_table": "authapp_user_token",
            },
        ),
        migrations.AddConstraint(
            model_name="userotp",
            constraint=models.CheckConstraint(
                check=models.Q(
                    ("phone__isnull", False), ("email__isnull", False), _connector="OR"
                ),
                name="phone_and_email_not_both_null_otp",
            ),
        ),
        migrations.AddConstraint(
            model_name="customuser",
            constraint=models.CheckConstraint(
                check=models.Q(
                    ("phone__isnull", False), ("email__isnull", False), _connector="OR"
                ),
                name="phone_and_email_not_both_null_user",
            ),
        ),
    ]
