import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { ReactiveFormsModule } from '@angular/forms';
import { ActivatedRoute } from '@angular/router';

import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { TranslatePipe } from '@ngx-translate/core';
import { Observable, finalize } from 'rxjs';

import { DataUtils, FileLoadError } from 'app/core/util/data-util.service';
import { EventManager, EventWithContent } from 'app/core/util/event-manager.service';
import { AlertError } from 'app/shared/alert/alert-error';
import { AlertErrorModel } from 'app/shared/alert/alert-error.model';
import { TranslateDirective } from 'app/shared/language';
import { UserPreferenceService } from '../service/user-preference.service';
import { IUserPreference } from '../user-preference.model';

import { UserPreferenceFormGroup, UserPreferenceFormService } from './user-preference-form.service';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector: 'jhi-user-preference-update',
  templateUrl: './user-preference-update.html',
  imports: [TranslateDirective, TranslatePipe, FontAwesomeModule, AlertError, ReactiveFormsModule],
})
export class UserPreferenceUpdate implements OnInit {
  readonly isSaving = signal(false);
  userPreference: IUserPreference | null = null;

  protected dataUtils = inject(DataUtils);
  protected eventManager = inject(EventManager);
  protected userPreferenceService = inject(UserPreferenceService);
  protected userPreferenceFormService = inject(UserPreferenceFormService);
  protected activatedRoute = inject(ActivatedRoute);

  // eslint-disable-next-line @typescript-eslint/member-ordering
  editForm: UserPreferenceFormGroup = this.userPreferenceFormService.createUserPreferenceFormGroup();

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(({ userPreference }) => {
      this.userPreference = userPreference;
      if (userPreference) {
        this.updateForm(userPreference);
      }
    });
  }

  byteSize(base64String: string): string {
    return this.dataUtils.byteSize(base64String);
  }

  openFile(base64String: string, contentType: string | null | undefined): void {
    this.dataUtils.openFile(base64String, contentType);
  }

  setFileData(event: Event, field: string, isImage: boolean): void {
    this.dataUtils.loadFileToForm(event, this.editForm, field, isImage).subscribe({
      error: (err: FileLoadError) =>
        this.eventManager.broadcast(
          new EventWithContent<AlertErrorModel>('jobSearchRustApp.error', { ...err, key: `error.file.${err.key}` }),
        ),
    });
  }

  previousState(): void {
    globalThis.history.back();
  }

  save(): void {
    this.isSaving.set(true);
    const userPreference = this.userPreferenceFormService.getUserPreference(this.editForm);
    if (userPreference.id === null) {
      this.subscribeToSaveResponse(this.userPreferenceService.create(userPreference));
    } else {
      this.subscribeToSaveResponse(this.userPreferenceService.update(userPreference));
    }
  }

  protected subscribeToSaveResponse(result: Observable<IUserPreference | null>): void {
    result.pipe(finalize(() => this.onSaveFinalize())).subscribe({
      next: () => this.onSaveSuccess(),
      error: () => this.onSaveError(),
    });
  }

  protected onSaveSuccess(): void {
    this.previousState();
  }

  protected onSaveError(): void {
    // Api for inheritance.
  }

  protected onSaveFinalize(): void {
    this.isSaving.set(false);
  }

  protected updateForm(userPreference: IUserPreference): void {
    this.userPreference = userPreference;
    this.userPreferenceFormService.resetForm(this.editForm, userPreference);
  }
}
