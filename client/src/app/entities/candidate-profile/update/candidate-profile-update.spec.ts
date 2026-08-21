import { beforeEach, describe, expect, it, vitest } from 'vitest';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ActivatedRoute } from '@angular/router';

import { provideTranslateService } from '@ngx-translate/core';
import { Subject, from, of } from 'rxjs';

import { ICandidateProfile } from '../candidate-profile.model';
import { CandidateProfileService } from '../service/candidate-profile.service';

import { CandidateProfileFormService } from './candidate-profile-form.service';
import { CandidateProfileUpdate } from './candidate-profile-update';

describe('CandidateProfile Management Update Component', () => {
  let comp: CandidateProfileUpdate;
  let fixture: ComponentFixture<CandidateProfileUpdate>;
  let activatedRoute: ActivatedRoute;
  let candidateProfileFormService: CandidateProfileFormService;
  let candidateProfileService: CandidateProfileService;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        provideTranslateService(),
        provideHttpClientTesting(),
        {
          provide: ActivatedRoute,
          useValue: {
            params: from([{}]),
          },
        },
      ],
    });

    fixture = TestBed.createComponent(CandidateProfileUpdate);
    activatedRoute = TestBed.inject(ActivatedRoute);
    candidateProfileFormService = TestBed.inject(CandidateProfileFormService);
    candidateProfileService = TestBed.inject(CandidateProfileService);

    comp = fixture.componentInstance;
  });

  describe('ngOnInit', () => {
    it('should update editForm', () => {
      const candidateProfile: ICandidateProfile = { id: 10019 };

      activatedRoute.data = of({ candidateProfile });
      comp.ngOnInit();

      expect(comp.candidateProfile).toEqual(candidateProfile);
    });
  });

  describe('save', () => {
    it('should call update service on save for existing entity', () => {
      // GIVEN
      const saveSubject = new Subject<ICandidateProfile>();
      const candidateProfile = { id: 25911 };
      vitest.spyOn(candidateProfileFormService, 'getCandidateProfile').mockReturnValue(candidateProfile);
      vitest.spyOn(candidateProfileService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ candidateProfile });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(candidateProfile);
      saveSubject.complete();

      // THEN
      expect(candidateProfileFormService.getCandidateProfile).toHaveBeenCalled();
      expect(comp.previousState).toHaveBeenCalled();
      expect(candidateProfileService.update).toHaveBeenCalledWith(expect.objectContaining(candidateProfile));
      expect(comp.isSaving()).toEqual(false);
    });

    it('should call create service on save for new entity', () => {
      // GIVEN
      const saveSubject = new Subject<ICandidateProfile>();
      const candidateProfile = { id: 25911 };
      vitest.spyOn(candidateProfileFormService, 'getCandidateProfile').mockReturnValue({ id: null });
      vitest.spyOn(candidateProfileService, 'create').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ candidateProfile: null });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(candidateProfile);
      saveSubject.complete();

      // THEN
      expect(candidateProfileFormService.getCandidateProfile).toHaveBeenCalled();
      expect(candidateProfileService.create).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).toHaveBeenCalled();
    });

    it('should set isSaving to false on error', () => {
      // GIVEN
      const saveSubject = new Subject<ICandidateProfile>();
      const candidateProfile = { id: 25911 };
      vitest.spyOn(candidateProfileService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ candidateProfile });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.error('This is an error!');

      // THEN
      expect(candidateProfileService.update).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).not.toHaveBeenCalled();
    });
  });
});
