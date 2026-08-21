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
import { ICandidateProfile } from '../candidate-profile.model';
import { CandidateProfileService } from '../service/candidate-profile.service';

import { CandidateProfileFormGroup, CandidateProfileFormService } from './candidate-profile-form.service';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector: 'jhi-candidate-profile-update',
  templateUrl: './candidate-profile-update.html',
  imports: [TranslateDirective, TranslatePipe, FontAwesomeModule, AlertError, ReactiveFormsModule],
})
export class CandidateProfileUpdate implements OnInit {
  readonly isSaving = signal(false);
  candidateProfile: ICandidateProfile | null = null;

  protected dataUtils = inject(DataUtils);
  protected eventManager = inject(EventManager);
  protected candidateProfileService = inject(CandidateProfileService);
  protected candidateProfileFormService = inject(CandidateProfileFormService);
  protected activatedRoute = inject(ActivatedRoute);

  // eslint-disable-next-line @typescript-eslint/member-ordering
  editForm: CandidateProfileFormGroup = this.candidateProfileFormService.createCandidateProfileFormGroup();

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(({ candidateProfile }) => {
      this.candidateProfile = candidateProfile;
      if (candidateProfile) {
        this.updateForm(candidateProfile);
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
    const candidateProfile = this.candidateProfileFormService.getCandidateProfile(this.editForm);
    if (candidateProfile.id === null) {
      this.subscribeToSaveResponse(this.candidateProfileService.create(candidateProfile));
    } else {
      this.subscribeToSaveResponse(this.candidateProfileService.update(candidateProfile));
    }
  }

  protected subscribeToSaveResponse(result: Observable<ICandidateProfile | null>): void {
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

  protected updateForm(candidateProfile: ICandidateProfile): void {
    this.candidateProfile = candidateProfile;
    this.candidateProfileFormService.resetForm(this.editForm, candidateProfile);
  }
}
